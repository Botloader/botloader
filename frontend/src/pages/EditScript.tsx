import { BotGuild, isErrorResponse, Plugin, Script, ScriptsWithPlugins } from "botloader-common";
import { useRef, useState } from "react";
import { useSession } from "../components/Session";
import "./EditScript.css";
import { AsyncOpButton } from "../components/AsyncOpButton";
import { debugMessageStore } from "../misc/DebugMessages";
import { DevConsole } from "../components/DevConsole";
import { createFetchDataContext, FetchDataGuarded, useFetchedDataBehindGuard } from "../components/FetchData";
import { Box, Chip, Divider, ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { ScriptingIde } from "../components/ScriptIde";
import { BlLink } from "../components/BLLink";

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
    const [diffSource, setDiffSource] = useState<"unsaved" | "pluginPublished" | null>(null);

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
        await session.apiClient.reloadGuildVm(props.guild.guild.id);
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

        await session.apiClient.reloadGuildVm(props.guild.guild.id);
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
                <Typography>Editing script</Typography><Chip variant="outlined" label={props.script.name} />
                <BlLink to={`/servers/${props.guild.guild.id}`}>Back</BlLink>
                <Divider sx={{ mb: 1 }} />
                <Typography>Diff mode</Typography>
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
                >
                    {props.script.plugin_id !== null ? <ToggleButton value="pluginPublished">Latest Plugin</ToggleButton> : null}
                    <ToggleButton value="unsaved">Saved</ToggleButton>
                    <ToggleButton value="off">Off</ToggleButton>
                </ToggleButtonGroup>
                <Divider sx={{ mb: 1 }} />
                <Typography>Script is {props.script.enabled ? <span className="status-good">Enabled</span> : <span className="status-bad">Disabled</span>}</Typography>
                {props.script.enabled ?
                    <AsyncOpButton className="primary" label="Disable" onClick={() => toggleScript(props.script.id, false)}></AsyncOpButton>
                    :
                    <AsyncOpButton className="primary" label="Enable" onClick={() => toggleScript(props.script.id, true)}></AsyncOpButton>
                }
                <Divider sx={{ mb: 1 }} />
                {isDirty ?
                    <AsyncOpButton className="primary" label="Save" onClick={() => save(newSource.current)}></AsyncOpButton>
                    : <p>No changes made</p>}
            </Box>
            <Box sx={{ overflowY: "auto" }}>
                <DevConsole guildId={props.guild.guild.id} />
            </Box>
        </ScriptingIde >
    </Box>
}
