import { BotGuild, isErrorResponse, Plugin, Script, ScriptsWithPlugins } from "botloader-common";
import { useRef, useState } from "react";
import "./EditScript.css";
import { AsyncOpButton } from "../../../../../../components/AsyncOpButton";
import { debugMessageStore } from "../../../../../../misc/DebugMessages";
import { DevConsole } from "../../../../../../components/DevConsole";
import { createFetchDataContext, FetchDataGuarded, useFetchedDataBehindGuard } from "../../../../../../components/FetchData";
import { Alert, Box, Chip, Divider, Switch, ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { ScriptingIde } from "../../../../../../components/ScriptIde";
import { BlLink } from "../../../../../../components/BLLink";
import { useSession } from "../../../../../../modules/session/useSession";

export const scriptsContext = createFetchDataContext<ScriptsWithPlugins>();

export function EditScriptPage(props: { guild: BotGuild, scriptId: number }) {
    const session = useSession();

    async function fetchScripts() {
        let resp = await session.apiClient.getAllScriptsWithPlugins(props.guild.guild.id);
        return resp;
    }

    return <FetchDataGuarded loader={fetchScripts} context={scriptsContext}>
        <InnerPage guild={props.guild} scriptId={props.scriptId} />
    </FetchDataGuarded>
}

export function InnerPage(props: { guild: BotGuild, scriptId: number }) {
    const { value: scripts } = useFetchedDataBehindGuard(scriptsContext);
    const script = scripts.scripts.find((v) => v.id === props.scriptId);
    const plugin = script?.plugin_id !== null ? scripts.plugins.find((v) => v.id === script?.plugin_id) : undefined;

    if (!script) {
        return <>Unknown script</>
    } else if (script.plugin_id && !plugin) {
        return <>Unknown plugin</>
    } else {
        return <LoadedNew guild={props.guild} script={script} plugin={plugin}></LoadedNew>
    }
}

function LoadedNew(props: { guild: BotGuild, script: Script, plugin?: Plugin }) {
    const { value: scripts, setData } = useFetchedDataBehindGuard(scriptsContext);
    const session = useSession();
    const [isDirty, setIsDirty] = useState(false);
    const newSource = useRef(props.script.original_source);
    const [diffSource, setDiffSource] = useState<"unsaved" | "pluginPublished" | null>(() => {
        let query = new URLSearchParams(window.location.search);
        let initialDiffMode = query.get("diffMode");
        return initialDiffMode as any;
    });

    async function setScript(newScript: Script) {
        setData((current) => {
            let scriptsCop = [...(current?.scripts ?? [])]
            let index = scriptsCop.findIndex((v) => v.id === newScript.id);
            if (index >= 0) {
                scriptsCop[index] = newScript;
            }

            return {
                plugins: current?.plugins ?? [],
                scripts: scriptsCop,
            };
        })
    }

    async function toggleScript(scriptId: number, enabled: boolean) {
        let newScript = await session.apiClient.updateScript(props.guild.guild.id, scriptId, {
            enabled,
        });
        if (!isErrorResponse(newScript)) {
            setScript(newScript);
        }
    }

    async function save(content: string) {
        debugMessageStore.pushMessage({
            guildId: props.guild.guild.id,
            level: "Client",
            message: "Saving..."
        });

        console.log("Saving!");

        let updated = await session.apiClient.updateScript(props.guild.guild.id, props.script.id, {
            original_source: content,
        });
        if (!isErrorResponse(updated)) {
            setScript(updated);
        }

        debugMessageStore.pushMessage({
            guildId: props.guild.guild.id,
            level: "Client",
            message: "Changes are live!"
        });

        setIsDirty(false);
    }

    return <Box display={"flex"} flexGrow="1">
        <ScriptingIde
            initialSource={props.script.original_source}
            onSave={save}
            files={scripts.scripts.filter((v) => v.id !== props.script.id && v.plugin_id === null)
                .map((v) => ({ name: v.name, content: v.original_source }))}
            isDiffEditor={diffSource !== null}
            diffSource={diffSource === "unsaved" ? props.script.original_source ?? "" : props.plugin?.data.published_version ?? ""}
            onChange={(source) => {
                newSource.current = source ?? "";
                if (source !== props.script.original_source) {
                    if (!isDirty) {
                        setIsDirty(true);
                    }
                } else {
                    if (isDirty) {
                        setIsDirty(false);
                    }
                }
            }}

        >
            <Box p={1}>
                <Box display={"flex"} justifyContent={"space-around"} >
                    <Chip variant="outlined" label={props.script.name + ".ts"} />
                    <Switch checked={props.script.enabled} disabled={false} color={"success"} onChange={(evt) => {
                        toggleScript(props.script.id, evt.target.checked)
                    }} />
                </Box>
                {!props.script.enabled &&
                    <Alert severity="warning" >This script is not enabled</Alert>
                }

                <Divider sx={{ mb: 1 }} />

                <BlLink fullWidth to={`/servers/${props.guild.guild.id}`}>Back to server</BlLink>

                <Divider sx={{ mb: 1 }} />

                <Typography align="center">Diff mode</Typography>
                <ToggleButtonGroup
                    color="primary"
                    value={diffSource === null ? "off" : diffSource}
                    exclusive
                    onChange={(_, value) => {
                        if (value === "off") {
                            setDiffSource(null)
                        } else {
                            setDiffSource(value as "unsaved" | "pluginPublished");
                        }
                    }}
                    aria-label="Diff mode"
                    fullWidth
                    size="small"
                >
                    {props.script.plugin_id !== null ? <ToggleButton value="pluginPublished">Latest Plugin</ToggleButton> : null}
                    <ToggleButton value="unsaved">Saved</ToggleButton>
                    <ToggleButton value="off">Off</ToggleButton>
                </ToggleButtonGroup>
                <Divider sx={{ mb: 1 }} />
                {isDirty ?
                    <><Typography>Changes have been made <AsyncOpButton className="primary" label="Save" onClick={() => save(newSource.current)}></AsyncOpButton></Typography></>
                    : <Typography>No changes made</Typography>}
                <Divider sx={{ mb: 1 }} />
            </Box>
            <Box sx={{ overflowY: "auto" }}>
                <DevConsole guildId={props.guild.guild.id} />
            </Box>
        </ScriptingIde >
    </Box>
}
