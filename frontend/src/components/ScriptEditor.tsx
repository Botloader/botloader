import { useEffect, useRef } from "react";
import Editor, { DiffEditor } from "@monaco-editor/react";
import monaco from "monaco-editor";
import { useBotloaderMonaco } from "./BotloaderSdk";

const DEFAULT_EMPTY_SCRIPT_CONTENT =
    `// import { Commands, Discord, HttpClient, Tasks } from 'botloader';
import {} from 'botloader';

// Type in the script content here
// ctrl-s to save, changes will go live after that
// Newly created scripts are disabled, you can enable it in the sidebar
// You can find a lot of script examples in the support server
// Docs are located at: https://botloader.io/docs/
// There's also more in depth guides available at: https://botloader.io/book/

// Example command:
// script.createCommand(Commands.slashCommand("echo", "I respond with what you said")
//     .addOptionString("what", "what to echo")
//     .build(async (ctx, args) => {
//         const what = args.what;
//         await ctx.createFollowup(\`echo response: \${what}\`);
//     })
// )
`

export function ScriptEditor(props: {
    onSave: (content: string) => any,
    onChange?: (content: string | undefined) => any,
    initialSource?: string,
    originalDiffSource?: string,
    isDiffEditor?: boolean,
    files?: IncludeFile[],
    isReadOnly?: boolean,
}) {
    const monacoRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
    const blSdkInit = useBotloaderMonaco(props.files);

    // we track it separately because otherwise it would clear when swapping between diff and code editor
    // 
    // the proper way is probably saving the model but ehh
    const editedValue = useRef(props.initialSource || DEFAULT_EMPTY_SCRIPT_CONTENT);

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
        await monacoRef.current?.getAction('editor.action.formatDocument')?.run()
        const value = monacoRef.current?.getValue() || "";
        let innerIsDirty = value !== props.initialSource

        if (!innerIsDirty || isSaving) {
            return;
        }

        await props.onSave(value);
    }

    function onValueChange(value: string | undefined) {
        editedValue.current = value || "";
        if (props.onChange) {
            props.onChange(value);
        }
    }

    if (!blSdkInit) {
        return <>Loading...</>
    }

    if (props.isDiffEditor) {
        return <DiffEditor
            modified={editedValue.current || ""}
            modifiedModelPath="file:///temp.ts"
            original={props.originalDiffSource}
            originalModelPath="file://temp_og.ts"
            originalLanguage="typescript"
            modifiedLanguage="typescript"
            theme="vs-dark"
            onMount={handleEditorDidMountDiff}
            options={{ readOnly: props.isReadOnly }}
            key="diff-editor"
            keepCurrentOriginalModel={true}
            keepCurrentModifiedModel={true}
        />

    } else {
        return <Editor
            path="file:///some_script.ts"
            theme="vs-dark"
            defaultLanguage="typescript"
            defaultValue={editedValue.current}
            saveViewState={false}
            onChange={onValueChange}
            onMount={handleEditorDidMount}
            options={{ readOnly: props.isReadOnly }}
            key="normal-editor"
        />
    }

}

export interface IncludeFile {
    name: string,
    content: string,
}


