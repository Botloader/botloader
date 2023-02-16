import { useEffect, useRef, useState } from "react";
import { BotGuild, GuildMetaConfig, GuildPremiumSlot, isErrorResponse, Script } from "botloader-common";
import { useCurrentGuild } from "../components/GuildsProvider";
import { useSession } from "../components/Session";
import { AsyncOpButton } from "../components/AsyncOpButton";
import { BuildConfig } from "../BuildConfig";
import { Navigate, useNavigate, useParams } from "react-router-dom";
import { Panel } from "../components/Panel";
import { SideNav } from "../components/SideNav";
import { EditScriptPage } from "./EditScript";
import { Alert, Box, Button, Paper, Stack, Switch, Typography } from "@mui/material";

export function GuildPagesWrapper({ children }: { children: React.ReactNode }) {
    let guild = useCurrentGuild();
    if (guild) {
        if (guild.connected) {
            return <div className="guild-page">
                {children}
            </div>
        } else {
            return <div className="page-wrapper">
                <InviteGuildPage guild={guild} />
            </div>
        }
    } else {
        return <div className="page-wrapper">
            <NoGuildPage />
        </div>
    }
}


function InviteGuildPage(props: { guild: BotGuild }) {
    return <a href={`https://discord.com/api/oauth2/authorize?client_id=${BuildConfig.botloaderClientId}&permissions=515463572672&scope=bot%20applications.commands&guild_id=${props.guild.guild.id}`} className="add-to-server" target="_blank" rel="noreferrer">Click here to add to server!</a>;
}

function NoGuildPage() {
    return <p>That's and unknown guild m8</p>
}

export function GuildSideNav() {
    const guild = useCurrentGuild();

    const navItems = {
        "home": {
            label: "Home",
            isNavLink: true,
            exact: true,
            path: `/servers/${guild?.guild.id}/`,
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
    console.log(scriptId);

    return <EditScriptPage guild={guild} scriptId={parseInt(scriptId!)}></EditScriptPage>
}


export function GuildHome() {
    const guild = useCurrentGuild()!;

    return <Stack spacing={2}>
        <Alert severity="warning">This is a reminder that this service is currently in an early beta state and everything you're seeing is in a unfinished state, especially when it comes to this website.</Alert>
        <PremiumPanel guild={guild}></PremiumPanel>
        <GuildScripts guild={guild}></GuildScripts>
    </Stack>
}

function PremiumPanel(props: { guild: BotGuild }) {
    const [slots, setSlots] = useState<GuildPremiumSlot[] | undefined | null>(undefined);
    const session = useSession();

    useEffect(() => {
        loadConfig();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [props, session])

    async function loadConfig() {
        let resp = await session.apiClient.getGuildPremiumSlots(props.guild.guild.id);
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
                : slots === undefined ? <p>Loading...</p>
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
    const [scripts, setScripts] = useState<Script[] | undefined | null>(undefined);
    const session = useSession();
    const createScriptInput = useRef<HTMLInputElement>(null);
    const [createError, setCreateError] = useState("");
    const [scriptCreated, setScriptCreated] = useState<Script | null>(null)

    async function loadScripts() {
        let resp = await session.apiClient.getAllScripts(props.guild.guild.id);
        if (isErrorResponse(resp)) {
            setScripts(null);
        } else {
            setScripts(resp);
        }
    }

    useEffect(() => {
        loadScripts();

        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [props, session])

    async function delScript(scriptId: number) {
        let resp = await session.apiClient.delScript(props.guild.guild.id, scriptId);
        if (!isErrorResponse(resp)) {
            await loadScripts();
        }

        await session.apiClient.reloadGuildVm(props.guild.guild.id);
    }

    async function toggleScript(scriptId: number, enabled: boolean) {
        let resp = await session.apiClient.updateScript(props.guild.guild.id, scriptId, {
            enabled,
        });
        if (!isErrorResponse(resp)) {
            let newScripts = [...scripts ?? []];
            let script = newScripts.find(v => v.id === scriptId);
            if (script) {
                script.enabled = resp.enabled;
            }
            setScripts(newScripts);
        }

        await session.apiClient.reloadGuildVm(props.guild.guild.id);
    }

    async function createScript() {
        setCreateError("");

        let name = createScriptInput.current?.value;
        if (name && name.length > 0) {
            console.log("Creating script, value:", createScriptInput.current?.value);
            let resp = await session.apiClient.createScript(props.guild.guild.id, {
                enabled: false,
                name: name,
                original_source: "",
            })

            if (isErrorResponse(resp)) {
                setCreateError(JSON.stringify(resp))
            } else {
                setScriptCreated(resp);
            }
        }
    }

    return <>
        <Typography variant="h4">Create a new script</Typography>
        <Paper sx={{ p: 1 }}>
            <div className="create-script">
                <input type="text" ref={createScriptInput}></input>
                <AsyncOpButton label="Create" onClick={createScript}></AsyncOpButton>
            </div>
            {createError ? <p>Error creating script: <code>{createError}</code></p> : null}
            {scriptCreated ? <Navigate to={`/servers/${props.guild.guild.id}/scripts/${scriptCreated.id}/edit`}></Navigate> : null}
        </Paper >
        <Typography variant="h4" sx={{ ml: 1, mb: 1 }}>Scripts</Typography>
        <Paper>
            {scripts ?
                <Stack spacing={1}>
                    {scripts.map(script => <ScriptItem key={script.id}
                        script={script}
                        guildId={props.guild.guild.id}
                        toggleScript={toggleScript}
                        deleteScript={delScript} />)}
                </Stack>
                : scripts === null
                    ? <p>Failed loading scripts</p>
                    : <p>Loading...</p>
            }
        </Paper>
    </>
}

function ScriptItem({ script, guildId, toggleScript, deleteScript }: {
    script: Script,
    guildId: string,
    toggleScript: (id: number, on: boolean) => any,
    deleteScript: (id: number) => any
}) {
    const navigate = useNavigate();
    const [isToggling, setToggling] = useState(false);

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

    return <Box sx={{
        p: 1, display: "flex", alignItems: "center",
        borderLeft: 2,
        borderColor: script.enabled ? "success.dark" : "error.dark",
        '&:hover': { backgroundColor: "action.hover" }
    }}>
        <Typography variant="body1" flexGrow={1}>{script.name}.ts</Typography>
        <Stack direction={"row"}>
            <Button onClick={() => navigate(`/servers/${guildId}/scripts/${script.id}/edit`)}>
                Edit
            </Button>

            <Switch checked={script.enabled} disabled={isToggling} color={"success"} onChange={(evt) => {
                toggleWrapper(evt.target.checked)
            }} />
            <AsyncOpButton label="delete" onClick={() => deleteConfirm()}></AsyncOpButton>
        </Stack>
    </Box>
}