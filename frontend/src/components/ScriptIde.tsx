import { Box, Typography } from "@mui/material";
import { ReactNode } from "react";
import { DevConsole } from "./DevConsole";
import { IncludeFile, ScriptEditor } from "./ScriptEditor";

type Props = {
    children: ReactNode,
    onSave: (content: string) => any,
    initialSource?: string,
    diffSource?: string,
    isDiffEditor: boolean,
    readOnly?: boolean,
    files?: IncludeFile[],
    onChange?: (content: string | undefined) => any,
}

export function ScriptingIde(props: Props) {
    return <>
        <Box sx={{ display: "flex", flexDirection: "row", flexGrow: 1 }}>
            <Box sx={{ flexGrow: 1, marginRight: "300px" }}>
                <ScriptEditor
                    initialSource={props.initialSource}
                    onSave={props.onSave}
                    files={props.files}
                    isDiffEditor={props.isDiffEditor}
                    originalDiffSource={props.diffSource}
                    onChange={props.onChange}
                />
            </Box>
            <Box width={300} display="flex" flexDirection="column" position={"absolute"} top={69} bottom={0} right={0}>
                {props.children}
                {/* <Box sx={{ overflowY: "auto" }}>
                    <DevConsole />
                </Box> */}
            </Box>
        </Box >
    </>
}