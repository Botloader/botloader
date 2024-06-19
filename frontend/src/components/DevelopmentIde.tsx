import { Box } from "@mui/material"
import { ReactNode, useEffect, useState } from "react"
import { IncludeFile, ScriptEditor } from "./ScriptEditor"
import { DevConsole } from "./DevConsole"

type Props = {
    children: ReactNode,
    onSave: (content: string) => any,
    isDiffEditor: boolean,
    customDiffContent?: string,
    isReadyOnly?: boolean,
    files: IncludeFile[],
    selectedFile: string,
    onChange?: (content: string | undefined) => any,
    consoleGuildId?: string
}

export function DevelopmentIde(props: Props) {
    const [resizingConsole, setResizingConsole] = useState<{ startPosition: number, target: any, pageYStart: number } | null>(null)
    const [consoleHeight, setConsoleHeight] = useState(200)

    useEffect(() => {
        if (!resizingConsole) {
            return
        }

        function onMouseMove(evt: MouseEvent) {
            let delta = resizingConsole!.pageYStart - evt.pageY
            setConsoleHeight(resizingConsole!.startPosition + delta)
        }

        function onMouseUp(evt: MouseEvent) {
            setResizingConsole(null)
        }

        document.addEventListener("mousemove", onMouseMove)
        document.addEventListener("mouseup", onMouseUp)

        return () => {
            document.removeEventListener("mousemove", onMouseMove)
            document.removeEventListener("mouseup", onMouseUp)
        }
    }, [resizingConsole])

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
            <Box sx={{ flexGrow: 1, textWrap: "stable" }} display={"flex"} flexDirection={"column"} minHeight={0} alignItems={"stretch"} >
                <Box sx={{ flexGrow: 1, minHeight: 100, flexBasis: 0 }}>
                    <ScriptEditor
                        // initialSource={props.initialSource}
                        onSave={props.onSave}
                        files={props.files}
                        selectedFileName={props.selectedFile}
                        isDiffEditor={props.isDiffEditor}
                        customDiffContent={props.customDiffContent}
                        // originalDiffSource={props.diffSource}
                        onChange={props.onChange}
                        isReadOnly={props.isReadyOnly}
                    />
                </Box>
                {props.consoleGuildId &&
                    <Box height={consoleHeight + "px"} maxWidth={"100%"} display={"flex"} flexDirection={"column"}>
                        <div
                            style={{
                                backgroundColor: "black",
                                cursor: "row-resize",
                                width: "100%",
                                height: "5px",
                                margin: 0,
                            }}
                            onMouseDown={(evt) => {
                                console.log(evt)
                                setResizingConsole({ startPosition: consoleHeight, target: evt.target, pageYStart: evt.pageY })
                            }}
                        ></div>
                        <Box sx={{ overflowY: "auto", overflowX: "hidden", width: "calc(100vw - 300px)" }} maxHeight={"100%"} minWidth={0} flexGrow={1} flexBasis={0}>
                            <DevConsole guildId={props.consoleGuildId} />
                        </Box>
                    </Box>
                }
            </Box>
        </Box >
    </>
}