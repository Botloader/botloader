import { BotGuild, isErrorResponse, Plugin, Script, ScriptsWithPlugins } from "botloader-common";
import { useCallback, useMemo, useRef, useState } from "react";
import "./EditScript.css";
import { AsyncOpButton } from "../../../../../../components/AsyncOpButton";
import { debugMessageStore } from "../../../../../../misc/DebugMessages";
import { createFetchDataContext, FetchDataGuarded, useFetchedDataBehindGuard } from "../../../../../../components/FetchData";
import { Alert, Box, Chip, Divider, Switch, ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { BlLink } from "../../../../../../components/BLLink";
import { useSession } from "../../../../../../modules/session/useSession";
import { guildScriptToFile } from "../../../../../../components/ScriptEditor";
import { DevelopmentIde } from "../../../../../../components/DevelopmentIde";
import { ScriptEnabledIndicator } from "../../../../../../components/GuildSideNav";
import { useNavigate } from "react-router-dom";
import { useCurrentGuildId } from "../../../../../../modules/guilds/CurrentGuild";

export const scriptsContext = createFetchDataContext<ScriptsWithPlugins>();

export function EditScriptPage(props: { guild: BotGuild, scriptId: number }) {
    const session = useSession();

    const guildId = props.guild.guild.id
    const fetchScripts = useCallback(async () => {
        let resp = await session.apiClient.getAllScriptsWithPlugins(guildId);
        return resp;
    }, [guildId, session.apiClient])

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

    const [dirtyFiles, setDirtyFiles] = useState([] as number[])
    const isDirty = dirtyFiles.includes(props.script.id)

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

        setDirtyFiles((current) => (current.filter(v => v !== props.script.id)))
    }

    const files = useMemo(() => {
        return scripts.scripts.map(guildScriptToFile)
    }, [scripts])

    return <Box display={"flex"} flexGrow="1">
        <DevelopmentIde
            onSave={save}
            files={files}
            isDiffEditor={diffSource !== null}
            selectedFile={props.script.name}
            onChange={(source) => {
                newSource.current = source ?? "";
                if (source !== props.script.original_source) {
                    if (!isDirty) {
                        setDirtyFiles((current) => ([...current, props.script.id]))
                    }
                } else {
                    if (isDirty) {
                        setDirtyFiles((current) => (current.filter(v => v !== props.script.id)))
                    }
                }
            }}
            consoleGuildId={props.guild.guild.id}
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
                <Typography p={1} variant="overline" color="grey">Scripts</Typography>
                {scripts.scripts.filter(v => v.plugin_id === null).map(v => (
                    <SidebarScriptItem script={v} key={v.id} dirty={dirtyFiles.includes(v.id)} selected={v.id === props.script.id} />
                ))}

                <Divider sx={{ mb: 1 }} />

                <Typography variant="overline" color={"grey"} p={1}>Plugins</Typography>

                {scripts.scripts.filter(v => v.plugin_id !== null).map(v => (
                    <SidebarScriptItem script={v} plugin={scripts.plugins.find(p => p.id === v.plugin_id)} key={v.id} dirty={dirtyFiles.includes(v.id)} selected={v.id === props.script.id} />
                ))}

            </Box>
        </DevelopmentIde>
    </Box>
}

function SidebarScriptItem(props: { script: Script, plugin?: Plugin, dirty: boolean, selected: boolean }) {
    const navigate = useNavigate()
    const guildId = useCurrentGuildId()

    return <Box display={"flex"} p={1} alignItems={"center"} gap={1} sx={{
        ":hover": {
            backgroundColor: "rgba(0,0,0,0.5)",
            cursor: "pointer"
        },
        backgroundColor: props.selected ? "rgba(255,255,255,0.2)" : "",
    }} onClick={() => {
        navigate(`/servers/${guildId}/scripts/${props.script.id}/edit`)
    }}>
        <ScriptEnabledIndicator enabled={props.script.enabled}></ScriptEnabledIndicator>
        <Typography>{props.script.name}</Typography>
        {props.dirty ? <Chip size="small" label="unsaved"></Chip> : null}
    </Box>
}
