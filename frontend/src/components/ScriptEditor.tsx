import { useEffect, useRef } from "react";
import Editor, { DiffEditor } from "@monaco-editor/react";
import monaco from "monaco-editor";
import { useBotloaderMonaco } from "./BotloaderSdk";
import { Loading } from "./Loading";
import { Plugin, Script } from "botloader-common";

const DEFAULT_EMPTY_SCRIPT_CONTENT =
    `import { Commands, Discord, HttpClient, Tasks } from 'botloader';

// Type in the script content here
// ctrl-s to save, changes will go live after that
// Newly created scripts are disabled, you can enable it in the sidebar
// You can find a lot of script examples in the support server
// Docs are located at: https://botloader.io/docs/
// There's also more in depth guides available at: https://botloader.io/book/

// Example command:
// script.createSlashCommand("echo", "I respond with what you said")
//     .addOptionString("what", "what to echo")
//     .build(async (ctx, args) => {
//         const what = args.what;
//         await ctx.createFollowup(\`echo response: \${what}\`);
//     })
`

export function ScriptEditor(props: {
    onSave: (content: string) => any,
    onChange?: (content: string | undefined) => any,
    isDiffEditor?: boolean,
    customDiffContent?: string,
    files: IncludeFile[],
    selectedFileName: string,
    isReadOnly?: boolean,
}) {
    const monacoRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
    const blSdkInit = useBotloaderMonaco(props.files);

    // we track it separately because otherwise it would clear when swapping between diff and code editor
    // 
    // the proper way is probably saving the model but ehh

    const selectedFile = props.files?.find(v => v.name === props.selectedFileName)
    // const editedValue = useRef(props.initialSource || DEFAULT_EMPTY_SCRIPT_CONTENT);

    useEffect(() => {
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

        document.addEventListener("keydown", handleKeyDown);
        document.addEventListener("keyup", handleKeyUp);
        return () => {
            document.removeEventListener("keydown", handleKeyDown);
            document.removeEventListener("keyup", handleKeyUp);
        }
    })

    useEffect(() => {
        function onResize() {
            monacoRef.current?.layout()
        }

        window.addEventListener("resize", onResize)

        return () => {
            window.removeEventListener("resize", onResize)
        }
    }, [monacoRef])

    if (!selectedFile) {
        return <>Couldn't find file</>
    }

    function handleEditorDidMount(editor: monaco.editor.IStandaloneCodeEditor) {
        // here is another way to get monaco instance
        // you can also store it in `useRef` for further usage
        monacoRef.current = editor;
    }

    function handleEditorDidMountDiff(editor: monaco.editor.IStandaloneDiffEditor) {
        // here is another way to get monaco instance
        // you can also store it in `useRef` for further usage
        const modifiedEditor = editor.getModifiedEditor();
        monacoRef.current = modifiedEditor;

        modifiedEditor.onDidChangeModelContent(() => onValueChange(modifiedEditor.getValue()));
    }

    let isSaving = false;
    async function save() {
        if (!monacoRef.current) {
            return
        }

        await monacoRef.current.getAction('editor.action.formatDocument')?.run()
        const value = monacoRef.current.getValue();

        let innerIsDirty = value !== selectedFile?.content

        if (!innerIsDirty || isSaving) {
            return;
        }

        await props.onSave(value);
    }

    function onValueChange(value: string | undefined) {
        // editedValue.current = value || "";
        if (props.onChange) {
            props.onChange(value);
        }
    }

    if (!blSdkInit) {
        return <Loading />
    }

    if (props.isDiffEditor) {
        return <DiffEditor
            // modified={editedValue.current || ""}
            modifiedModelPath={"file:///" + selectedFile.name + ".ts"}
            original={(props.customDiffContent ?? selectedFile.content) || DEFAULT_EMPTY_SCRIPT_CONTENT}
            // originalModelPath="file://temp_og.ts"
            originalLanguage="typescript"
            modifiedLanguage="typescript"
            theme="vs-dark"
            onMount={handleEditorDidMountDiff}
            options={{
                readOnly: props.isReadOnly,
                inlayHints: {
                    enabled: "on",
                    padding: true,
                    fontSize: 10,
                }
            }}
            key={"diff-editor-" + props.customDiffContent ?? selectedFile.content}
            keepCurrentOriginalModel={false}
            keepCurrentModifiedModel={true}
        />

    } else {
        return <Editor
            path={"file:///" + selectedFile.name + ".ts"}
            // path="file:///some_script.ts"
            theme="vs-dark"
            defaultLanguage="typescript"
            defaultValue={DEFAULT_EMPTY_SCRIPT_CONTENT}
            saveViewState={false}
            onChange={onValueChange}
            onMount={handleEditorDidMount}
            options={{
                readOnly: props.isReadOnly,
                inlayHints: {
                    enabled: "on",
                    padding: true,
                    fontSize: 10,
                },
            }}
            key="normal-editor"
            keepCurrentModel={true}
        />
    }

}

export interface IncludeFile {
    name: string,
    content: string,
}

export function guildScriptToFile(guildScript: Script): IncludeFile {
    if (guildScript.plugin_id !== null) {
        return {
            content: guildScript.original_source || DEFAULT_EMPTY_SCRIPT_CONTENT,
            name: "plugins/" + guildScript.plugin_id
        }
    } else {
        return {
            content: guildScript.original_source || DEFAULT_EMPTY_SCRIPT_CONTENT,
            name: guildScript.name
        }
    }
}

export function pluginToFiles(plugin: Plugin): { published: IncludeFile, dev: IncludeFile } {
    return {
        dev: {
            name: "plugins-dev/" + plugin.id,
            content: plugin.data.dev_version || DEFAULT_EMPTY_SCRIPT_CONTENT,
        },

        published: {
            name: "plugins-published/" + plugin.id,
            content: plugin.data.published_version || DEFAULT_EMPTY_SCRIPT_CONTENT,
        }
    }
}

