// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import { ApiClient, isErrorResponse, UserGuild } from 'botloader-common';

import { tmpdir } from 'os';
import { mkdtemp } from 'fs/promises';
import { join } from 'path';
import { WorkspaceManager } from './workspacemanager';
import { BotloaderWS, LogItem } from './ws';
import { BotloaderSourceControl, CHANGED_FILES_SCM_GROUP } from './guildspace';
import { createFetcher } from './util';
import fetch from 'node-fetch';
import * as tar from "tar";


// this method is called when your extension is activated 
// your extension is activated the very first time the command is executed
export async function activate(context: vscode.ExtensionContext) {

	let outputChannel = vscode.window.createOutputChannel("Botloader");
	context.subscriptions.push(outputChannel);

	let token = await context.secrets.get("botloader-api-key");

	const config = vscode.workspace.getConfiguration("botloader");
	const apiBase: string = config.get("apiHost")!;
	const apiHttps: boolean = config.get("apiHttpsEnabled")!;

	const httpApiBase = apiHttps ? "https://" + apiBase : "http://" + apiBase;
	const wsApiBase = apiHttps ? "wss://" + apiBase : "ws://" + apiBase;

	let ws = new BotloaderWS(wsApiBase, handleLogMessage, token);
	let apiClient = new ApiClient(createFetcher(), httpApiBase, token);

	let manager = new WorkspaceManager(apiClient, ws, outputChannel);
	context.subscriptions.push(manager);

	await updateTypeDecls(context);

	context.subscriptions.push(vscode.commands.registerCommand('botloader-vscode.setup-workspace', async () => {
		// The code you place here will be executed every time your command is executed
		// Display a message box to the user
		let resp = await apiClient.getCurrentUserGuilds();
		if (isErrorResponse(resp)) {
			vscode.window.showErrorMessage("Invalid token:" + JSON.stringify(resp));
			return;
		}

		let typeSelection = await vscode.window.showQuickPick(["Temp folder", "Pick folder"], {
			canPickMany: false,
			title: "Workspace folder",
		});

		let folder: vscode.Uri | undefined = undefined;
		if (typeSelection === "Temp folder") {
			let tmpDir = await mkdtemp(join(tmpdir(), "botloader"));
			folder = vscode.Uri.parse("file:/" + tmpDir);
		} else {
			let selected = await vscode.window.showOpenDialog({
				canSelectFiles: false,
				canSelectFolders: true,
				canSelectMany: false,
				title: "Folder to set up guild workspace in",
			});
			if (selected && selected.length > 0) {
				folder = selected[0];
			}
		}

		if (!folder) {
			return;
		}

		const filtered = resp.guilds.filter(elem => elem.connected && hasAdmin(elem.guild));
		let selection = await vscode.window.showQuickPick(filtered.map(elem => `${elem.guild.name} (${elem.guild.id})`), {
			canPickMany: false,
			title: "Select server"
		});

		let selectedServer = filtered.find(elem => `${elem.guild.name} (${elem.guild.id})` === selection);
		if (!selectedServer) {
			vscode.window.showErrorMessage("Unknown server");
		}

		vscode.window.showInformationMessage(`selected as ${selection}`);

		await setupWorkspace(selectedServer!.guild, folder);

	}), vscode.commands.registerCommand('botloader-vscode.set-accesstoken', async () => {

		let key = await vscode.window.showInputBox({
			password: true,
			title: "API key",
		});

		if (!key) {
			return;
		}

		let newClient = new ApiClient(createFetcher(), httpApiBase, key);
		let resp = await newClient.getCurrentUser();

		if (isErrorResponse(resp)) {
			vscode.window.showErrorMessage("Invalid token:" + JSON.stringify(resp));
		} else {
			vscode.window.showInformationMessage(`Logged in as ${resp.username}#${resp.discriminator}`);
			apiClient.token = newClient.token;
			ws.setToken(newClient.token!);
			await context.secrets.store("botloader-api-key", key as string);
		}
	}), vscode.commands.registerCommand('botloader-vscode.push', async (arg: any) => {
		if (arg === undefined) {
			if (vscode.window.activeTextEditor) {
				await manager.pushUri(vscode.window.activeTextEditor.document.uri);
			}
		} else if (isScmGroup(arg)) {
			await manager.pushScmGroup(arg);
		} else if (containsResourceUri(arg)) {
			await manager.pushUri(arg.resourceUri);
		}
	}), vscode.commands.registerCommand('botloader-vscode.push-repo', async (arg: any) => {
		if (arg === undefined) {
			await manager.pushOneRepo();
		} else if (isScmProvider(arg)) {
			await manager.pushRepo(arg);
		}
	}), vscode.commands.registerCommand('botloader-vscode.sync', async (arg: any) => {
		if (arg === undefined) {
			await manager.syncOne();
		} else if (isScmProvider(arg)) {
			await manager.syncScm(arg);
		}
	}), vscode.commands.registerCommand('botloader-vscode.fetch-types', async (arg: any) => {
		await updateTypeDecls(context);
	}), vscode.commands.registerCommand('botloader-vscode.reset-tsconfig', async (arg: any) => {
		for (let workspace of manager.openGuildWorkspaces) {
			await writeTsConfig(context, workspace.folder);
		}

		vscode.commands.executeCommand("typescript.reloadProjects");
	}));

	async function setupWorkspace(guild: UserGuild, dirUri: vscode.Uri) {
		await vscode.workspace.fs.createDirectory(vscode.Uri.joinPath(dirUri, "/.botloader"));
		await vscode.workspace.fs.createDirectory(vscode.Uri.joinPath(dirUri, "/.botloader/scripts"));

		let textEncoder = new TextEncoder();
		await vscode.workspace.fs.writeFile(vscode.Uri.joinPath(dirUri, `/.botloader/index.json`), textEncoder.encode(JSON.stringify({
			guild: guild,
			openScripts: [],
		})));

		await writeTsConfig(context, dirUri);

		vscode.workspace.updateWorkspaceFolders(0, 0, {
			uri: dirUri,
			name: guild.name,
		});
	}

	function handleLogMessage(item: LogItem) {
		let tag = item.level;
		if (item.guild_id) {
			tag += " " + item.guild_id;
		}
		if (item.script_context) {
			tag += ` ${item.script_context.filename}`;
			if (item.script_context.line_col) {
				const [line, col] = item.script_context.line_col;
				tag += `:${line}:${col}`;
			}
		}

		let full = `[${tag}] ${item.message}`;
		outputChannel.appendLine(full);
	}
}

export async function writeTsConfig(context: vscode.ExtensionContext, workspace: vscode.Uri) {
	let textEncoder = new TextEncoder();
	await vscode.workspace.fs.writeFile(vscode.Uri.joinPath(workspace, `/tsconfig.json`), textEncoder.encode(JSON.stringify(generateTsConfig(context), undefined, 4)));
}

function generateTsConfig(context: vscode.ExtensionContext) {
	const typingsGlobal = vscode.Uri.joinPath(context.globalStorageUri, "/typings");

	const indexPath = vscode.Uri.joinPath(typingsGlobal, "/index").fsPath;
	const globalsPath = vscode.Uri.joinPath(typingsGlobal, "/globals/").fsPath;

	return {
		"include": [
			"*.ts",
			globalsPath,
		],
		"compilerOptions": {
			"module": "ES2020",
			"noImplicitAny": true,
			"removeComments": true,
			"preserveConstEnums": true,
			"sourceMap": false,
			"target": "esnext",
			"alwaysStrict": true,
			"strict": true,
			"strictNullChecks": true,
			"baseUrl": "./",
			"lib": ["esnext"],
			"paths": {
				"botloader": [
					indexPath
				]
			}
		}
	};
}

// this method is called when your extension is deactivated
export function deactivate() { }

const permAdmin = BigInt("0x0000000008");
const permManageServer = BigInt("0x0000000020");

function hasAdmin(g: UserGuild): boolean {
	if (g.owner) {
		return true;
	}


	const perms = BigInt(g.permissions);
	if ((perms & permAdmin) === permAdmin) {
		return true;
	}

	if ((perms & permManageServer) === permManageServer) {
		return true;
	}

	return false;
}

interface BotloaderJson {
	guild: UserGuild,
	openScripts: number[],
}

function isScmGroup(arg: any): arg is vscode.SourceControlResourceGroup {
	if ((arg as vscode.SourceControlResourceGroup).id === CHANGED_FILES_SCM_GROUP) {
		return true;
	}

	return false;
}


interface ResourceUriContainer {
	resourceUri: vscode.Uri
}

function containsResourceUri(arg: any): arg is ResourceUriContainer {
	if ((arg as ResourceUriContainer).resourceUri !== undefined) {
		return true;
	}

	return false;
}

function isScmProvider(arg: any): arg is BotloaderSourceControl {
	let cast = arg as BotloaderSourceControl;
	if (cast.isBotloaderSourceControl !== undefined && cast.isBotloaderSourceControl) {
		return true;
	}

	return false;
}

// syncs the extensions tyep decls to the global extension folder
// the extentionUri is version specific so we can't reference it in tsconfig's
// (also in the future we might want to downlaod typedecls from)
async function updateTypeDecls(context: vscode.ExtensionContext) {
	const typingsUriGlobal = vscode.Uri.joinPath(context.globalStorageUri, "/typings");
	try {
		// clear the typings folder first
		await vscode.workspace.fs.delete(typingsUriGlobal, {
			recursive: true,
			useTrash: false,
		});
	} catch { }
	await vscode.workspace.fs.createDirectory(typingsUriGlobal);

	await downloadTypeDecls(context);

	// const extensionTypings = vscode.Uri.joinPath(context.extensionUri, "/out/typings");
	// // await vscode.workspace.fs.createDirectory(typingsUriGlobal);
	// await vscode.workspace.fs.copy(extensionTypings, typingsUriGlobal);
	// console.log(typingsUriGlobal);
}

async function downloadTypeDecls(context: vscode.ExtensionContext): Promise<void> {
	let resp = await fetch("https://botloader.io/typings.tar");

	const outPath = vscode.Uri.joinPath(context.globalStorageUri, "/typings");
	console.log("downloading types to: ", outPath.fsPath, resp.status);

	let stream = tar.extract({
		cwd: outPath.fsPath,
		strip: 1,
	});

	let body = resp.body?.pipe(stream);

	stream.on("end", () => {
		vscode.commands.executeCommand("typescript.reloadProjects");
	});
}