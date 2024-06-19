import { Box } from "@mui/material";
import { ReactNode } from "react";
import { IncludeFile, ScriptEditor } from "./ScriptEditor";

type Props = {
    children: ReactNode,
    onSave: (content: string) => any,
    // initialSource?: string,
    // diffSource?: string,
    isDiffEditor: boolean,
    isReadyOnly?: boolean,
    files: IncludeFile[],
    selectedFileName: string,
    onChange?: (content: string | undefined) => any,
}

export function ScriptingEditorWrapper(props: Props) {
    return <>
        <Box sx={{ display: "flex", flexDirection: "row", alignContent: "stretch", flexGrow: 1 }}>
            <Box
                width={300}
                display="flex"
                flexDirection="column"
                height={"0px"}
                minHeight={"100%"}
            >
                {props.children}
            </Box>
            <Box sx={{ flexGrow: 1 }}>
                <ScriptEditor
                    // initialSource={props.initialSource}
                    onSave={props.onSave}
                    files={props.files}
                    selectedFileName={props.selectedFileName}
                    isDiffEditor={props.isDiffEditor}
                    // originalDiffSource={props.diffSource}
                    onChange={props.onChange}
                    isReadOnly={props.isReadyOnly}
                />
            </Box>
        </Box >
    </>
}