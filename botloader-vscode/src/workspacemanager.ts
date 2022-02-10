import * as vscode from 'vscode';
import { ApiClient } from 'botloader-common';
import { BotloaderSourceControl, GuildScriptWorkspace } from './guildspace';
import { IndexFile } from './models';
import { BotloaderWS } from './ws';

export class WorkspaceManager implements vscode.Disposable {
    openGuildWorkspaces: GuildScriptWorkspace[] = [];
    apiClient: ApiClient;
    ws: BotloaderWS;
    outputChannel: vscode.OutputChannel;

    otherDisposables: vscode.Disposable[] = [];

    constructor(apiClient: ApiClient, ws: BotloaderWS, outputChannel: vscode.OutputChannel) {
        this.apiClient = apiClient;
        this.ws = ws;
        this.outputChannel = outputChannel;

        this.otherDisposables.push(vscode.workspace.onDidChangeWorkspaceFolders(this.workspaceFoldersChange.bind(this)));

        for (let folder of vscode.workspace.workspaceFolders || []) {
            this.checkFolder(folder.uri);
        }
    }

    async workspaceFoldersChange(evt: vscode.WorkspaceFoldersChangeEvent) {
        for (let added of evt.added) {
            await this.checkFolder(added.uri);
        }

        for (let removed of evt.removed) {
            const index = this.openGuildWorkspaces.findIndex(e => e.folder === removed.uri);
            if (index !== -1) {
                const elem = this.openGuildWorkspaces[index];
                elem.dispose();
                this.openGuildWorkspaces.splice(index, 1);
            }
        }
    }

    async checkFolder(folder: vscode.Uri) {
        try {
            // should throw an error if it dosen't exist
            let indexFile = await vscode.workspace.fs.readFile(vscode.Uri.joinPath(folder, "/.botloader/index.json"));
            let decoder = new TextDecoder("utf-8");
            let parsedIndex: IndexFile = JSON.parse(decoder.decode(indexFile));

            this.ws.subscribeGuild(parsedIndex.guild.id);

            if (!this.openGuildWorkspaces.some(elem => elem.folder === folder)) {
                this.openGuildWorkspaces.push(new GuildScriptWorkspace(folder, this.apiClient));
            }
            this.outputChannel.show(true);
        } catch { }
    }

    async pushUri(uri: vscode.Uri) {
        let folder = vscode.workspace.getWorkspaceFolder(uri);
        if (folder) {
            let uriString = folder.uri.toString();
            let guildSpace = this.openGuildWorkspaces.find(e => e.folder.toString() === uriString);
            if (guildSpace) {
                guildSpace.pushUri(uri);
            }
        }
    }

    async pushScmGroup(group: vscode.SourceControlResourceGroup) {
        let aFile = group.resourceStates[0].resourceUri;
        let folder = vscode.workspace.getWorkspaceFolder(aFile);
        if (folder) {
            let uriString = folder.uri.toString();
            let guildSpace = this.openGuildWorkspaces.find(e => e.folder.toString() === uriString);
            if (guildSpace) {
                guildSpace.pushScmGroup(group);
            }
        }
    }

    async syncScm(control: vscode.SourceControl) {
        let root = control.rootUri;
        let workspace = this.openGuildWorkspaces.find(ws => ws.folder.toString() === root?.toString());
        if (workspace) {
            await workspace.syncWorkspaceWithProgress();
        }
    }

    async syncOne() {
        if (this.openGuildWorkspaces.length > 0) {
            await this.openGuildWorkspaces[0].syncWorkspaceWithProgress();
        }
    }

    async pushOneRepo() {
        if (this.openGuildWorkspaces.length > 0) {
            await this.openGuildWorkspaces[0].pushAll();
        }
    }

    async pushRepo(repo: BotloaderSourceControl) {
        const guildSpace = this.openGuildWorkspaces.find(e => e.folder.toString() === repo.rootUri?.toString());
        if (guildSpace) {
            await guildSpace.pushAll();
        }
    }

    dispose() {
        for (let dis of this.otherDisposables) {
            dis.dispose();
        }

        for (let guild of this.openGuildWorkspaces) {
            guild.dispose();
        }
    }
}

