import { useEffect, useRef, useState } from "react";
import { BotGuild, GuildMetaConfig, GuildPremiumSlot, isErrorResponse, Plugin, Script } from "botloader-common";
import { AsyncOpButton } from "../../../components/AsyncOpButton";
import { useNavigate, useParams } from "react-router-dom";
import { Panel } from "../../../components/Panel";
import { SideNav } from "../../../components/SideNav";
import { EditScriptPage } from "./scripts/[script_id]/edit/EditScript";
import { Alert, AlertTitle, Box, Button, Paper, Stack, Switch, TextField, Typography } from "@mui/material";
import { FetchDataGuard } from "../../../components/FetchData";
import { BlLink } from "../../../components/BLLink";
import { UseNotifications } from "../../../components/Notifications";
import { useSession } from "../../../modules/session/useSession";
import { useCurrentGuild } from "../../../modules/guilds/CurrentGuild";
import { guildScriptsContext, useCurrentGuildScripts } from "../../../modules/guilds/GuildScriptsProvider";
import { Loading } from "../../../components/Loading";

export function GuildSideNav() {
    const guild = useCurrentGuild();

    const navItems = {
        "home": {
            label: "Home",
            isNavLink: true,
            exact: true,
            path: `/servers/${guild?.value?.guild.id}`,
        },
        // "scripts": {
        //     label: "Scripts",
        //     isNavLink: true,
        //     exact: true,
        //     path: `/servers/${props.guild.guild.id}/scripts`,
        // },
        // "settings": {
        //     label: "Settings",
        //     isNavLink: true,
        //     exact: true,
        //     path: `/servers/${props.guild.guild.id}/settings`,
        // },
    }

    return <SideNav items={navItems}></SideNav>
}

export function EditGuildScript() {
    const guild = useCurrentGuild()!;

    let { scriptId } = useParams();

    return <EditScriptPage guild={guild!.value!} scriptId={parseInt(scriptId!)}></EditScriptPage>
}

export function GuildHome() {
    const guild = useCurrentGuild()!;

    return <Stack spacing={2}>
        <Alert severity="warning">This is a reminder that this service is currently in an early beta state and everything you're seeing is in a unfinished state, especially when it comes to this website.</Alert>
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


    return <>
        <Typography variant="h4">Premium/Lite</Typography>
        <Paper sx={{ p: 1 }}>
            {slots === null ? <p>Failed loading slots</p>
                : slots === undefined ? <Loading />
                    : slots.length > 0 ? slots.map(v => <div className="guild-premium-slots" key={v.id}>{v.tier} by <code>{v.user_id}</code></div>)
                        : <p>This server is on the free plan</p>}
        </Paper>
    </>
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
    const [isToggling, setToggling] = useState(false);
    const session = useSession();
    const notifications = UseNotifications();
    const { reload } = useCurrentGuildScripts();

    async function toggleWrapper(on: boolean) {
        setToggling(true);
        await toggleScript(script.id, on);
        try {

        } finally {
            setToggling(false);
        }
    }

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
        p: 1, display: "flex", alignItems: "center",
        borderLeft: 2,
        borderColor: script.enabled ? "success.dark" : "error.dark",
        '&:hover': { backgroundColor: "action.hover" }
    }}>
        <Typography variant="body1" flexGrow={1}>{script.name}.ts</Typography>
        <Stack direction={"row"} alignItems="center">
            {plugin ? (
                <>
                    <Typography variant="body1">{plugin.current_version === script.plugin_version_number ?
                        "Using latest version"
                        : script.plugin_version_number === null
                            ? "Using a modified version"
                            : "New version available"}</Typography>
                    <AsyncOpButton disabled={plugin.current_version === script.plugin_version_number} onClick={updatePluginVersion} label="Update version"></AsyncOpButton>
                    <BlLink disabled={plugin.current_version === script.plugin_version_number}
                        to={`/servers/${guildId}/scripts/${script.id}/edit?diffMode=pluginPublished`}>View changes</BlLink>
                </>
            ) : null}
            <BlLink to={`/servers/${guildId}/scripts/${script.id}/edit`}>
                Edit
            </BlLink>

            <Switch checked={script.enabled} disabled={isToggling} color={"success"} onChange={(evt) => {
                toggleWrapper(evt.target.checked)
            }} />
            <AsyncOpButton label="delete" onClick={() => deleteConfirm()}></AsyncOpButton>
        </Stack>
    </Box>
}