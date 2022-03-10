import { BotGuild, isErrorResponse, Script } from "botloader-common";
import { useEffect, useRef, useState } from "react";
import { useSession } from "../components/Session";
import "./EditScript.css";
import Editor, { useMonaco } from "@monaco-editor/react";
import untar from "js-untar";
import { AsyncOpButton } from "../components/AsyncOpButton";
import { GuildMessage, GuildMessages } from "../misc/GuildMessages";
import { WebsocketSession } from "../misc/WebsocketController";

const DEFAULT_EMPTY_SCRIPT_CONTENT =
    `import {} from 'botloader';

// Type in the script content here
// ctrl-s to save, changes will go live after that
// Newly created scripts are disabled, you can enable it in the sidebar
// You can find a lot of script examples in the support server
// Docs are located at: https://botloader.io/docs/
`

export function EditScriptPage(props: { guild: BotGuild, scriptId: number }) {
    const [script, setScript] = useState<Script | undefined | null>(undefined);
    const [scripts, setScripts] = useState<Script[] | undefined | null>(undefined);
    const [typings, setTypings] = useState<File[] | undefined | null>(undefined);
    const [hasSetTypings, setHasSetTypings] = useState(false);
    const monaco = useMonaco();
    const session = useSession();

    async function load() {
        await loadScripts()
        await loadTypings();
    }

    async function loadScripts() {
        let resp = await session.apiClient.getAllScripts(props.guild.guild.id);
        if (isErrorResponse(resp)) {
            setScript(null);
            setScripts(null);
        } else {
            let s = resp.find(v => v.id === props.scriptId);
            setScripts(resp);
            if (s) {
                setScript(s)
            } else {
                setScript(null);
                setScripts(null);
            }
        }
    }

    async function loadTypings() {
        let files = await downloadTypeDecls();
        setTypings(files);
    }

    useEffect(() => {
        load()
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [props, session])

    useEffect(() => {
        if (typings && monaco) {
            monaco.languages.typescript.typescriptDefaults.setExtraLibs(
                [
                    ...typings.filter(v => v.type === "0")
                        .map(v => {
                            // console.log(v.name);
                            return {
                                content: v.readAsString(),
                                filePath: "file:///" + v.name,
                            }
                        }),
                    ...(scripts?.filter(v => v.id !== script?.id)
                        .map(v => {
                            return {
                                content: v.original_source,
                                filePath: "file:///" + v.name + ".ts"
                            }
                        }) ?? [])
                ]
            )

            monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
                // typeRoots: ["typings/"],
                moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
                // baseUrl: "typings/",
                module: monaco.languages.typescript.ModuleKind.ESNext,
                // This property seems to fuck shit up, no idea why
                // lib: [
                //     "ES5",
                //     "ES2015",
                //     "ES2016",
                //     "ES2017",
                //     "ES2018",
                //     "ES2019",
                //     "ES2020",
                //     "ES2021",
                // ],
                "noImplicitAny": true,
                "removeComments": true,
                "preserveConstEnums": true,
                "sourceMap": false,
                "target": monaco.languages.typescript.ScriptTarget.ES2020,
                "alwaysStrict": true,
                "strict": true,
                "strictNullChecks": true,

                paths: {
                    "botloader": ["file:///typings/index.d.ts"]
                }
            })

            setHasSetTypings(true);
            console.log("set extra libs!");
        }

        // This is probably fine because of the if statement
        // also we don't need this to update
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [monaco, typings]);

    if (script && typings && hasSetTypings) {
        return <Loaded guild={props.guild} script={script} files={typings} refreshScripts={loadScripts}></Loaded>
    } else {
        return <ul>
            <li>Typings: {typings ? "Loaded" : typings === undefined ? "Loading..." : "Failed loading"}</li>
            <li>Scripts: {script ? "Loaded" : script === undefined ? "Loading..." : "Failed loading"}</li>
            <li>Set typings:: {hasSetTypings ? "true" : "false"}</li>
        </ul>
    }
}

function Loaded(props: { guild: BotGuild, script: Script, files: File[], refreshScripts: () => unknown }) {
    const session = useSession();
    const monaco = useMonaco();
    const currentValue = useRef(props.script.original_source);
    const [isDirty, setIsDirty] = useState(false);

    useEffect(() => {
        if (monaco) {
            console.log(monaco.languages.typescript.typescriptDefaults.getExtraLibs());
            console.log(monaco.languages.typescript.typescriptDefaults.getCompilerOptions());
            console.log(monaco.languages.typescript.typescriptDefaults.getEagerModelSync());
        }

    }, [monaco]);


    useEffect(() => {
        document.addEventListener("keydown", handleKeyDown);
        document.addEventListener("keyup", handleKeyUp);
        return () => {
            document.removeEventListener("keydown", handleKeyDown);
            document.removeEventListener("keyup", handleKeyUp);
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    })

    async function toggleScript(scriptId: number, enabled: boolean) {
        await session.apiClient.updateScript(props.guild.guild.id, scriptId, {
            enabled,
        });
        await props.refreshScripts();
        await session.apiClient.reloadGuildVm(props.guild.guild.id);
    }

    let ctrlSIsDown = false;
    function handleKeyDown(evt: KeyboardEvent) {
        if (evt.ctrlKey && evt.code === "KeyS") {
            evt.preventDefault();
            if (!ctrlSIsDown) {
                ctrlSIsDown = true;
                save();
            }
        }
    }
    function handleKeyUp(evt: KeyboardEvent) {
        if (evt.ctrlKey && evt.code === "KeyS") {
            ctrlSIsDown = false;
        }
    }

    let isSaving = false;
    async function save() {
        if (!isDirty || isSaving) {
            console.log(isDirty, isSaving);
            return;
        }
        GuildMessages.pushGuildMessage(props.guild.guild.id, {
            source: "Client",
            message: "Saving..."
        });

        console.log("Saving!");
        await session.apiClient.updateScript(props.guild.guild.id, props.script.id, {
            original_source: currentValue.current,
        });
        await props.refreshScripts();

        GuildMessages.pushGuildMessage(props.guild.guild.id, {
            source: "Client",
            message: "Successfully saved! Reloading guild vm..."
        });

        await session.apiClient.reloadGuildVm(props.guild.guild.id);
        GuildMessages.pushGuildMessage(props.guild.guild.id, {
            source: "Client",
            message: "Reloaded guild vm, changes are now live!"
        });

        setIsDirty(false);
    }

    function onvalueChange(value: string | undefined) {
        currentValue.current = value || "";
        if (currentValue.current !== props.script.original_source) {
            setIsDirty(true);
        } else {
            setIsDirty(false);
        }
    }

    console.log("render")
    return <div className="scripting-ide">
        <Editor
            // loading
            // className="scripting-editor"
            path="file:///some_script.ts"
            width={"75%"}
            height={"calc(100vh - 55px)"}
            className="scripting-editor"
            theme="vs-dark"
            defaultLanguage="typescript"
            defaultValue={props.script.original_source || DEFAULT_EMPTY_SCRIPT_CONTENT}
            saveViewState={false}
            onChange={onvalueChange}
        // onMount={handleEditorDidMount}
        />
        {/* <div className="scripting-editor"> */}
        {/* <p>test</p> */}
        {/* </div> */}
        <div className="scripting-panel">
            <div className="scripting-actions">
                <h3 className="scripting-header">Editing {props.script.name}.ts</h3>
                <div className="actions-row">
                    <p>Script is {props.script.enabled ? <span className="status-good">Enabled</span> : <span className="status-bad">Disabled</span>}</p>
                    {props.script.enabled ?
                        <AsyncOpButton className="primary" label="Disable" onClick={() => toggleScript(props.script.id, false)}></AsyncOpButton>
                        :
                        <AsyncOpButton className="primary" label="Enable" onClick={() => toggleScript(props.script.id, true)}></AsyncOpButton>
                    }
                </div>
                <div className="actions-row">
                    {isDirty ?
                        <AsyncOpButton className="primary" label="Save" onClick={() => save()}></AsyncOpButton>
                        : <p>No changes made</p>}
                </div>
            </div>
            <div className="scripting-console">
                <GuildConsole guild={props.guild}></GuildConsole>
            </div>
        </div>
    </div>
}

async function downloadTypeDecls(): Promise<File[]> {
    let resp = await fetch("https://botloader-misc.us-east-1.linodeobjects.com/typings-master.tar");
    let res = await untar(await resp.arrayBuffer());
    return res
}

interface File {
    name: string,
    mode: string,
    type: string,

    readAsString(): string,
    readAsJSON(): unknown,
}

function GuildConsole(props: { guild: BotGuild }) {
    const [messages, setMessages] = useState<GuildMessage[]>([])
    const listenerId = useRef<undefined | number>(undefined);
    const bottom = useRef<HTMLLIElement>(null);


    useEffect(() => {
        let messages = GuildMessages.getGuildMessages(props.guild.guild.id);
        setMessages(messages);

        WebsocketSession.subscribeGuild(props.guild.guild.id);
    }, [props.guild.guild.id])

    useEffect(() => {
        listenerId.current = GuildMessages.addListener(props.guild.guild.id, onNewMessage);

        return () => {
            if (listenerId.current) {
                GuildMessages.removeListener(props.guild.guild.id, listenerId.current);
            }
        }
    })

    useEffect(() => {
        if (bottom.current) {
            bottom.current.scrollIntoView({ behavior: 'auto' })
        }
    })

    function onNewMessage(message: GuildMessage) {
        let newMessages = [
            ...messages,
            message
        ]
        setMessages(newMessages);
    }

    return <ul className="guild-console">
        {messages.map(v =>
            <li key={v.id} className="guild-console-message"><span className="guild-console-message-source">{v.source}:</span>{v.message}</li>
        )}
        <li ref={bottom}></li>
    </ul>

}