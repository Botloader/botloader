import { useEffect, useRef, useState } from "react";
import { BotGuild, GuildMetaConfig, GuildPremiumSlot, isErrorResponse, Plugin, Script } from "botloader-common";
import { AsyncOpButton } from "../../../components/AsyncOpButton";
import { useNavigate, useParams } from "react-router-dom";
import { Panel } from "../../../components/Panel";
import { EditScriptPage } from "./scripts/[script_id]/edit/EditScript";
import {
    Alert,
    AlertTitle,
    Box,
    Button,
    Paper,
    Stack,
    TextField,
    Typography
} from "@mui/material";
import { FetchDataGuard } from "../../../components/FetchData";
import { BlLink } from "../../../components/BLLink";
import { UseNotifications } from "../../../components/Notifications";
import { useSession } from "../../../modules/session/useSession";
import { useCurrentGuild } from "../../../modules/guilds/CurrentGuild";
import { guildScriptsContext, useCurrentGuildScripts } from "../../../modules/guilds/GuildScriptsProvider";
import { Loading } from "../../../components/Loading";
import { ScriptEnableToggle } from "../../../components/ScriptEnableToggle";

export function EditGuildScript() {
    const guild = useCurrentGuild()!;

    let { scriptId } = useParams();

    return <EditScriptPage guild={guild!.value!} scriptId={parseInt(scriptId!)}></EditScriptPage>
}

export function GuildHome() {
    const guild = useCurrentGuild()!;

    return <Stack spacing={2}>
        <PremiumPanel guildId={guild!.value!.guild!.id!}></PremiumPanel>
        <FetchDataGuard context={guildScriptsContext}><GuildScripts guild={guild!.value!}></GuildScripts></FetchDataGuard>
    </Stack >
}

function PremiumPanel(props: { guildId: string }) {
    const [slots, setSlots] = useState<GuildPremiumSlot[] | undefined | null>(undefined);
    const session = useSession();

    useEffect(() => {
        loadConfig(props.guildId);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [props.guildId, session])


    async function loadConfig(guildId: string) {
        let resp = await session.apiClient.getGuildPremiumSlots(guildId);
        if (isErrorResponse(resp)) {
            setSlots(null);
        } else {
            setSlots(resp);
        }
    }

    if (slots === null) {
        return <Typography>Failed loading premium slots</Typography>
    }

    if (slots === undefined) {
        return <Loading />
    }

    if (slots.length > 0) {
        return slots.map(v => <PremiumBy key={v.id} slot={v}></PremiumBy>)
    }

    return <Alert severity="info">
        <AlertTitle>This server is on the free tier</AlertTitle>
        Botloader is developed and run by a single person in their spare time, please consider lite or premium to support the project.<br /><br />
        <BlLink to="/user/premium" variant="contained" color="success">Check out Lite/Premium</BlLink>
    </Alert>
}

function PremiumBy({ slot }: { slot: GuildPremiumSlot }) {
    return <Alert color="success">
        This server is on the {slot.tier} tier, provided by user id {slot.user_id}
    </Alert>
}

function GuildSettings(props: { guild: BotGuild }) {
    const [config, setConfig] = useState<GuildMetaConfig | undefined | null>(undefined);
    const session = useSession();

    useEffect(() => {
        loadConfig();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [props, session])

    async function loadConfig() {
        let conf = await session.apiClient.getGuildMetaConfig(props.guild.guild.id);
        if (isErrorResponse(conf)) {
            setConfig(null);
        } else {
            setConfig(conf);
        }
    }

    if (config === undefined) {
        return <p>Loading settings...</p>
    } else if (config === null) {
        return <p>Failed loading settings.</p>
    } else {
        return <InnerGuildSettings guild={props.guild} settings={config}></InnerGuildSettings>
    }
}

function InnerGuildSettings(props: { guild: BotGuild, settings: GuildMetaConfig }) {
    return <Panel>
        <p>Error channel: <code>{props.settings.error_channel_id || "not set"}</code></p>
    </Panel>
}

function GuildScripts(props: { guild: BotGuild }) {
    const createScriptInput = useRef<HTMLInputElement>(null);
    const [createError, setCreateError] = useState("");
    const [isCreating, setCreating] = useState<boolean>(false)
    const navigate = useNavigate()

    const { value: scripts, toggleScript, createScript, delScript } = useCurrentGuildScripts();
    const pluginScripts = scripts!.scripts.filter((v) => v.plugin_id !== null)
    const normalScripts = scripts!.scripts.filter((v) => v.plugin_id === null)


    async function submitCreateScript() {
        setCreateError("");

        let name = createScriptInput.current?.value;
        if (name && name.length > 0) {
            let resp = await createScript(name)
            if (isErrorResponse(resp)) {
                const nameError = resp.getFieldError("name")
                if (nameError) {
                    setCreateError(nameError)
                } else {
                    setCreateError(JSON.stringify(resp))
                }
            } else {
                navigate(`/servers/${props.guild.guild.id}/scripts/${resp.id}/edit`)
            }
        }
    }

    return <>
        {(scripts?.plugins.length ?? 0) < 1 && (scripts?.scripts.length ?? 0) < 1 ? <GetStartedPanel></GetStartedPanel> : null}
        <Typography variant="h4">Create a new script</Typography>
        <Paper sx={{ p: 1 }}>
            {isCreating && <Loading />}
            <form onSubmit={(evt) => {
                evt.preventDefault()
                if (!isCreating) {
                    setCreating(true)
                    submitCreateScript().finally(() => setCreating(false))
                }
            }}>
                <Stack direction={"row"}>
                    <TextField label="Name" inputRef={createScriptInput} />
                    <Button type="submit" disabled={isCreating}>Create</Button>
                </Stack>
            </form>
            {createError && <Alert severity="error"><AlertTitle>Error creating script</AlertTitle>{createError}</Alert>}
        </Paper >
        <Typography variant="h4" sx={{ ml: 1, mb: 1 }}>Scripts</Typography>
        <Paper>
            <Stack spacing={1}>
                {normalScripts.length < 1 && <Typography padding={1}>No scripts have been added to this server yet.</Typography>}
                {normalScripts.map(script => <ScriptItem key={script.id}
                    script={script}
                    guildId={props.guild.guild.id}
                    toggleScript={toggleScript}
                    deleteScript={delScript} />)}
            </Stack>
        </Paper>

        <Typography variant="h4" sx={{ ml: 1, mb: 1 }}>Plugins</Typography>
        <Paper>
            <Stack spacing={1}>
                {pluginScripts.length < 1 && <Typography padding={1}>No plugins have been added to this server yet.</Typography>}
                {pluginScripts.map(script => <ScriptItem key={script.id}
                    script={script}
                    plugin={scripts!.plugins.find((p) => p.id === script.plugin_id)}
                    guildId={props.guild.guild.id}
                    toggleScript={toggleScript}
                    deleteScript={delScript} />)}
            </Stack>
        </Paper>
    </>
}

function ScriptItem({ script, guildId, toggleScript, deleteScript, plugin }: {
    script: Script,
    plugin?: Plugin,
    guildId: string,
    toggleScript: (id: number, on: boolean) => any,
    deleteScript: (id: number) => any
}) {
    const session = useSession();
    const notifications = UseNotifications();
    const { reload } = useCurrentGuildScripts();

    async function deleteConfirm() {
        if (window.confirm("are you sure you want to delete this script?")) {
            await deleteScript(script.id);
        }
    }

    async function updatePluginVersion() {
        const resp = await session.apiClient.updateScriptPlugin(guildId, script.id);
        if (isErrorResponse(resp)) {
            notifications.push({ class: "error", message: "failed updating plugin: " + resp.response?.description });
        } else {
            reload();
            notifications.push({ class: "success", message: "updated plugin" });
        }
    }

    return <Box sx={{
        display: "flex", alignItems: "center",
        borderLeft: 2,
        borderColor: script.enabled ? "success.dark" : "error.dark",
        '&:hover': { backgroundColor: "action.hover" }
    }}
        onClick={() => console.log("whooo")}
    >
        <BlLink
            to={`/servers/${guildId}/scripts/${script.id}`}
            sx={{ flexGrow: 1, textTransform: "none", color: "unset", textAlign: "left", justifyContent: "start", p: 2 }}
        >
            {script.name}.ts
            {/* <Typography variant="body1" flexGrow={1} color={"white"} textTransform={"unset"}>{script.name}.ts</Typography> */}
        </BlLink>
        <Stack direction={"row"} alignItems="center" onClick={(evt) => evt.stopPropagation()}>
            {plugin
                ? (<>
                    <Typography variant="body1">{plugin.current_version === script.plugin_version_number ?
                        "Using latest version"
                        : script.plugin_version_number === null
                            ? "Using a modified version"
                            : "New version available"}</Typography>
                    <AsyncOpButton disabled={plugin.current_version === script.plugin_version_number} onClick={updatePluginVersion} label="Update version"></AsyncOpButton>
                    <BlLink disabled={plugin.current_version === script.plugin_version_number}
                        to={`/servers/${guildId}/scripts/${script.id}/edit?diffMode=pluginPublished`}>View changes</BlLink>
                </>)
                : null
            }

            {plugin
                ? <BlLink to={`/servers/${guildId}/scripts/${script.id}`}>
                    Settings
                </BlLink>
                : <>
                    <BlLink to={`/servers/${guildId}/scripts/${script.id}/edit`}>
                        Edit
                    </BlLink>
                    <ScriptEnableToggle script={script} />
                    <AsyncOpButton label="delete" onClick={() => deleteConfirm()}></AsyncOpButton>
                </>
            }
        </Stack >
    </Box >
}

function GetStartedPanel() {
    return <Alert sx={{ padding: 1 }} severity="success">
        <AlertTitle>Welcome to Botloader!</AlertTitle>
        <Typography>Check out the<BlLink to="/book/" newTab skipClientRouting>Guides</BlLink>for a getting starting guide!</Typography>
        <Typography>Check out the<BlLink to="/plugins">plugins page</BlLink>to add pre-made plugins to your server</Typography>
        <Typography>Join the<BlLink to="https://discord.gg/GhUeYeekdu" newTab skipClientRouting>Discord Server</BlLink>to ask any questions, provide feedback or just talk to other people and the dev!</Typography>
    </Alert>
}