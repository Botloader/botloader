import { createHash } from 'crypto';
import * as vscode from 'vscode';
import { ApiClient, ApiResult, isErrorResponse, Script } from 'botloader-common';
import { relative } from 'path';
import { IndexFile } from './models';

export const CHANGED_FILES_SCM_GROUP = "changed";

export class GuildScriptWorkspace implements vscode.Disposable, vscode.FileDecorationProvider {
    folder: vscode.Uri;
    apiClient: ApiClient;

    disposables: vscode.Disposable[] = [];

    scm: BotloaderSourceControl;
    changedFilesGroup: vscode.SourceControlResourceGroup;

    onChangeFileDecosEmitter: vscode.EventEmitter<vscode.Uri>;

    constructor(folder: vscode.Uri, apiClient: ApiClient) {
        console.log("botloader: adding script workspace folder:", folder);
        this.folder = folder;
        this.apiClient = apiClient;

        this.scm = vscode.scm.createSourceControl("botloader", "BotLoader", this.folder) as BotloaderSourceControl;
        this.scm.isBotloaderSourceControl = true;

        this.scm.inputBox.visible = false;
        this.changedFilesGroup = this.scm.createResourceGroup(CHANGED_FILES_SCM_GROUP, "Changed scripts");

        const watcherPattern = new vscode.RelativePattern(this.folder, "*.ts");
        console.log("Watcing: ", watcherPattern);
        const watcher = vscode.workspace.createFileSystemWatcher(watcherPattern);
        console.log(watcher);

        this.onChangeFileDecosEmitter = new vscode.EventEmitter();

        this.disposables.push(watcher);
        this.disposables.push(vscode.window.registerFileDecorationProvider(this));

        this.scm.quickDiffProvider = {
            provideOriginalResource: ((uri: vscode.Uri, cancel: vscode.CancellationToken) => {
                return this.provideOriginalResource(uri, cancel);
            }).bind(this),
        };

        watcher.onDidChange(this.onFileDidhange.bind(this));
        watcher.onDidCreate(this.onFileDidCreate.bind(this));
        watcher.onDidDelete(this.onFileDidDelete.bind(this));

        this.syncWorkspace();
    }

    get onDidChangeFileDecorations(): vscode.Event<vscode.Uri | vscode.Uri[] | undefined> | undefined {
        return this.onChangeFileDecosEmitter.event;
    }

    provideFileDecoration(uri: vscode.Uri, token: vscode.CancellationToken): vscode.ProviderResult<vscode.FileDecoration> {
        const wsFolder = vscode.workspace.getWorkspaceFolder(uri);
        if (wsFolder?.uri.toString() !== this.folder.toString()) {
            return undefined;
        }

        const uriString = uri.toString();
        const state = this.changedFilesGroup.resourceStates.find(s => s.resourceUri.toString() === uriString);
        if (state) {
            let changeState = (state as ResourceState).state;
            return new vscode.FileDecoration(stateBadges[changeState], stateLabels[changeState], this.stateColor(changeState));
        } else {
            return null;
        }
    }

    async provideOriginalResource(uri: vscode.Uri, cancel: vscode.CancellationToken) {
        const relative = vscode.workspace.asRelativePath(uri, false);
        const indexPath = vscode.Uri.joinPath(this.folder, ".botloader/scripts/", relative + ".bloader");
        return indexPath;
    }

    dispose() {
        for (let dis of this.disposables) {
            dis.dispose();
        }
    }

    async initialScan() {
        console.log("performing full scan");
        let filesWorking = await vscode.workspace.fs.readDirectory(this.folder);
        filesWorking = filesWorking.filter(f => f[0].endsWith(".ts"));

        for (let file of filesWorking) {
            await this.checkWorkingFile(file[0]);
            this.onChangeFileDecosEmitter.fire(vscode.Uri.joinPath(this.folder, "/" + file[0]));
        }

        let filesIndex = await vscode.workspace.fs.readDirectory(vscode.Uri.joinPath(this.folder, "/.botloader/scripts"));
        const filesIndexNames = filesIndex.filter(f => f[0].endsWith(".ts.bloader")).map(f => f[0].slice(0, f[0].length - 8));
        let deletedFiles = filesIndexNames.filter(iff => !filesWorking.some(wf => wf[0] === iff));
        for (let file of deletedFiles) {
            this.setFileDeleted(vscode.Uri.joinPath(this.folder, "/" + file[0]));
        }
    }

    async checkWorkingFile(name: string) {
        let uri = vscode.Uri.joinPath(this.folder, "/" + name);

        switch (await this.fileChangeState(name)) {
            case ChangeState.created:
                this.setFileCreated(uri);
                break;
            case ChangeState.modified:
                this.setFileModified(uri);
                break;
            default:
                this.removeFileResourceState(uri);
                break;
        }
    }

    async fileChangeState(name: string) {
        let uri = vscode.Uri.joinPath(this.folder, "/" + name);
        // let stat = await vscode.workspace.fs.stat(uri);

        let uriIndex = vscode.Uri.joinPath(this.folder, "/.botloader/scripts/" + name + ".bloader");

        try {
            await vscode.workspace.fs.stat(uriIndex);
        } catch {
            return ChangeState.created;
        }


        let hashWorking = await this.hashFile(uri);
        let hashIndex = await this.hashFile(uriIndex);

        if (hashIndex !== hashWorking) {
            // modified
            console.log(name, "Changed");
            return ChangeState.modified;
        } else {
            // unchanged
            console.log(name + " is unmodified");
            return undefined;
        }
    }

    modifiedStateDeco: vscode.SourceControlResourceDecorations = { tooltip: "Modified", iconPath: new vscode.ThemeIcon("pulse", new vscode.ThemeColor(themeColorModifiedUri)) };
    createdStateDeco: vscode.SourceControlResourceDecorations = { tooltip: "Created", iconPath: new vscode.ThemeIcon("new-file", new vscode.ThemeColor(themeColorAddedUri)) };
    deletedStateDeco: vscode.SourceControlResourceDecorations = { tooltip: "Deleted", strikeThrough: true, iconPath: new vscode.ThemeIcon("trash", new vscode.ThemeColor(themeColorDeletedUri)) };

    setFileModified(uri: vscode.Uri) {
        this.setFileResourceState({ resourceUri: uri, decorations: this.modifiedStateDeco, state: ChangeState.modified });
    }

    setFileCreated(uri: vscode.Uri) {
        this.setFileResourceState({ resourceUri: uri, decorations: this.createdStateDeco, state: ChangeState.created });
    }

    setFileDeleted(uri: vscode.Uri) {
        this.setFileResourceState({ resourceUri: uri, decorations: this.deletedStateDeco, state: ChangeState.deleted });
    }

    setFileResourceState(state: ResourceState) {
        const current = this.changedFilesGroup.resourceStates.findIndex(c => {
            return c.resourceUri.toString() === state.resourceUri.toString();
        });
        if (current !== -1) {
            // we have to create a new array to presumably trigger the setter for the ui to update? 
            // idk, the docs are pretty shit and literally just tell you to look at a 2k+ line file with no comments and a bunch of 
            // internal concepts you have no idea about as an "example", honestly fuck off  
            let newArr = [...this.changedFilesGroup.resourceStates];
            newArr[current] = state;
            this.changedFilesGroup.resourceStates = newArr;

            this.onChangeFileDecosEmitter.fire(state.resourceUri);
            return;
        }

        this.changedFilesGroup.resourceStates = [...this.changedFilesGroup.resourceStates, state];
        this.onChangeFileDecosEmitter.fire(state.resourceUri);
    }


    removeFileResourceState(uri: vscode.Uri) {
        const index = this.changedFilesGroup.resourceStates.findIndex(u => u.resourceUri.toString() === uri.toString());
        if (index === -1) {
            return;
        }
        const newArr = [...this.changedFilesGroup.resourceStates];
        newArr.splice(index, 1);
        this.changedFilesGroup.resourceStates = newArr;
        this.onChangeFileDecosEmitter.fire(uri);
    }

    async onFileDidhange(uri: vscode.Uri) {
        const relativePath = relative(this.folder.path, uri.path);
        this.checkWorkingFile(relativePath);
    }


    async onFileDidCreate(uri: vscode.Uri) {
        const relativePath = relative(this.folder.path, uri.path);
        this.checkWorkingFile(relativePath);
    }


    async onFileDidDelete(uri: vscode.Uri) {
        const relativePath = relative(this.folder.path, uri.path);
        let indexPath = vscode.Uri.joinPath(this.folder, "/.botloader/scripts/" + relativePath + ".bloader");
        try {
            await vscode.workspace.fs.stat(indexPath);
            console.log("Deleted existing file");
            this.setFileDeleted(uri);
        } catch {
            return;
        }
    }

    async hashFile(file: vscode.Uri) {
        const contents = await vscode.workspace.fs.readFile(file);

        let hash = createHash("sha256");
        hash.update(contents);
        return hash.digest('hex');
    }




    themeColorAdded = new vscode.ThemeColor(themeColorAddedUri);
    themeColorModified = new vscode.ThemeColor(themeColorModifiedUri);
    themeColorDeleted = new vscode.ThemeColor(themeColorDeletedUri);

    stateColor(state: ChangeState) {
        switch (state) {
            case ChangeState.created:
                return this.themeColorAdded;
            case ChangeState.deleted:
                return this.themeColorDeleted;
            case ChangeState.modified:
                return this.themeColorModified;
        }
    }

    async pushUri(uri: vscode.Uri) {
        await vscode.window.withProgress({ location: vscode.ProgressLocation.SourceControl }, async progress => {
            let index = await this.readIndexFile();
            await this.pushSingleChange(uri, index);

            await this.syncWorkspace();
            await this.apiClient.reloadGuildVm(index.guild.id);
        });
    }

    async pushAll() {
        await vscode.window.withProgress({ location: vscode.ProgressLocation.SourceControl }, async progress => {
            let index = await this.readIndexFile();

            for (let state of this.changedFilesGroup.resourceStates) {
                await this.pushSingleChange(state.resourceUri, index);
            }

            await this.syncWorkspace();
            await this.apiClient.reloadGuildVm(index.guild.id);
        });
    }

    async pushScmGroup(group: vscode.SourceControlResourceGroup) {
        await vscode.window.withProgress({ location: vscode.ProgressLocation.SourceControl }, async progress => {
            let index = await this.readIndexFile();

            for (let state of this.changedFilesGroup.resourceStates) {
                await this.pushSingleChange(state.resourceUri, index);
            }

            await this.syncWorkspace();
            await this.apiClient.reloadGuildVm(index.guild.id);
        });
    }

    async pushSingleChange(uri: vscode.Uri, index: IndexFile) {
        const relativePath = relative(this.folder.path, uri.path);
        let nameNoTs = relativePath.slice(0, relativePath.length - 3);

        let resState = this.changedFilesGroup.resourceStates.find(e => e.resourceUri.toString() === uri.toString()) as ResourceState | undefined;
        if (!resState) {
            return;
        }

        let indexScript = index.openScripts.find(e => e.name === nameNoTs);

        let resp: ApiResult<any> = null;
        switch (resState.state) {
            case ChangeState.created:
                let contentsCreate = await vscode.workspace.fs.readFile(uri);
                resp = await this.apiClient.createScript(index.guild.id, {
                    enabled: true,
                    name: nameNoTs,
                    // eslint-disable-next-line @typescript-eslint/naming-convention
                    original_source: contentsCreate.toString(),
                });
                break;
            case ChangeState.deleted:
                if (!indexScript) {
                    return;
                }

                resp = await this.apiClient.delScript(index.guild.id, indexScript.id);
                break;
            case ChangeState.modified:
                if (!indexScript) {
                    return;
                }

                let contentsModify = await vscode.workspace.fs.readFile(uri);
                resp = await this.apiClient.updateScript(index.guild.id, indexScript.id, {
                    enabled: true,
                    name: nameNoTs,
                    // eslint-disable-next-line @typescript-eslint/naming-convention
                    original_source: contentsModify.toString(),
                });
                break;
        }

        if (isErrorResponse(resp)) {
            vscode.window.showErrorMessage("failed push:" + JSON.stringify(resp));
        }
    }

    async readIndexFile() {
        const indexPath = vscode.Uri.joinPath(this.folder, ".botloader/index.json");
        let contents = await vscode.workspace.fs.readFile(indexPath);
        let decoder = new TextDecoder("utf-8");
        let parsedIndex: IndexFile = JSON.parse(decoder.decode(contents));
        return parsedIndex;
    }

    // synchronizesa the workspaces with the remote scripts
    // tries to be as non destructive as possible
    // will only overwrite files unchaged against the old index 
    async syncWorkspace() {
        console.log("Syncing workspace");
        let index = await this.readIndexFile();

        // first make a list of files identical to their index variant
        let filesWorking = await vscode.workspace.fs.readDirectory(this.folder);
        filesWorking = filesWorking.filter(f => f[0].endsWith(".ts"));

        // holds the current working files identical to the index files
        // will are safe to overwrite these
        let identicalIndexFiles = await this.getIdenticalIndexFiles(filesWorking.map(e => e[0]));
        // strip .ts suffix for convenience
        let identicalIndexNames = identicalIndexFiles.map(e => e.slice(0, e.length - 3));

        let resp = await this.apiClient.getAllScripts(index.guild.id);
        if (isErrorResponse(resp)) {
            throw new Error("failed fetching scripts");
        }

        // last resort security check, see checkValidName for more info
        const wsFolder = this.folder;
        resp = resp.filter(script => checkValidName(wsFolder, script.name));

        // nuke old index
        await vscode.workspace.fs.delete(vscode.Uri.joinPath(this.folder, "/.botloader/scripts"), {
            recursive: true,
            useTrash: false,
        });

        // create new scripts index
        let textEncoder = new TextEncoder();
        for (let script of resp) {
            await vscode.workspace.fs.writeFile(vscode.Uri.joinPath(this.folder, `/.botloader/scripts/${script.name}.ts.bloader`), textEncoder.encode(script.original_source));
        }

        if (resp.length < 1) {
            // ensure the script folder is there
            await vscode.workspace.fs.createDirectory(vscode.Uri.joinPath(this.folder, "/.botloader/scripts"));
        }

        let newOpenScripts = resp.map(s => {
            return {
                id: s.id,
                name: s.name,
            };
        });

        // write the new index
        await vscode.workspace.fs.writeFile(vscode.Uri.joinPath(this.folder, `/.botloader/index.json`), textEncoder.encode(JSON.stringify({
            guild: index.guild,
            openScripts: newOpenScripts,
        })));

        // create new files
        let newScripts = resp.filter(e => !index.openScripts.some(os => e.id === os.id));
        for (let script of newScripts) {
            let uri = vscode.Uri.joinPath(this.folder, `/${script.name}.ts`);
            try {
                await vscode.workspace.fs.stat(uri);
            } catch {
                await vscode.workspace.fs.writeFile(uri, textEncoder.encode(script.original_source));
            }
        }

        // update previous identical files
        let toUpdate = resp.filter(s => identicalIndexNames.some(os => s.name === os));
        for (let script of toUpdate) {
            let uri = vscode.Uri.joinPath(this.folder, `/${script.name}.ts`);
            await vscode.workspace.fs.writeFile(uri, textEncoder.encode(script.original_source));
        }

        // AND FINALLY remove unchanged deleted files
        let toDel = identicalIndexNames.filter(f => !(resp as Script[]).some(s => f === s.name));
        for (let del of toDel) {
            let uri = vscode.Uri.joinPath(this.folder, `/${del}.ts`);
            await vscode.workspace.fs.delete(uri);
        }

        this.changedFilesGroup.resourceStates = [];
        await this.initialScan();
    }

    async syncWorkspaceWithProgress() {
        await vscode.window.withProgress({ location: vscode.ProgressLocation.SourceControl }, async progress => {
            await this.syncWorkspace();
        });
    }

    async getIdenticalIndexFiles(files: string[]) {
        let result = [];
        for (let name of files) {
            if (await this.fileChangeState(name) === undefined) {
                result.push(name);
            }
        }

        return result;
    }
}

enum ChangeState {
    created,
    modified,
    deleted,
};

// why don't i just put them in the initializer you ask? 
// because if we reorder the enum then it will be fucked
let stateBadges: string[] = [];
stateBadges[ChangeState.created] = "U";
stateBadges[ChangeState.modified] = "M";
stateBadges[ChangeState.deleted] = "D";

let stateLabels: string[] = [];
stateLabels[ChangeState.created] = "Untracked";
stateLabels[ChangeState.modified] = "Modied";
stateLabels[ChangeState.deleted] = "Deleted";

interface ResourceState extends vscode.SourceControlResourceState {
    state: ChangeState,
}

const themeColorAddedUri = "botloaderDecoration.untrackedResourceForeground";
const themeColorModifiedUri = "botloaderDecoration.modifiedResourceForeground";
const themeColorDeletedUri = "botloaderDecoration.deletedResourceForeground";


// while this isn't needed as the backend does verification, this is a last resort to make sure
// script names and use ../ to escape the workspace folder
function checkValidName(wsFolder: vscode.Uri, name: string) {
    const uri = vscode.Uri.joinPath(wsFolder, `/${name}.ts`);
    const resolved = vscode.workspace.getWorkspaceFolder(uri);
    if (resolved && resolved.uri.toString() === wsFolder.toString()) {
        // this is inside the workspace folder
        return true;
    }

    return false;
}

export interface BotloaderSourceControl extends vscode.SourceControl {
    isBotloaderSourceControl: true,
}