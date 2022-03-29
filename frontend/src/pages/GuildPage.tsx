import { useEffect, useRef, useState } from "react";
import { BotGuild, GuildMetaConfig, GuildPremiumSlot, isErrorResponse, Script } from "botloader-common";
import { useCurrentGuild } from "../components/GuildsProvider";
import { useSession } from "../components/Session";
import './GuildPage.css'
import { AsyncOpButton } from "../components/AsyncOpButton";
import { BuildConfig } from "../BuildConfig";
import { Link, Redirect, Route, Switch, useParams } from "react-router-dom";
import { Panel } from "../components/Panel";
import { SideNav } from "../components/SideNav";
import { EditScriptPage } from "./EditScript";

export function GuildPage() {
    let guild = useCurrentGuild();
    if (guild) {
        if (guild.connected) {
            return <GuildLoadedPage guild={guild}></GuildLoadedPage>
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



function GuildLoadedPage(props: { guild: BotGuild }) {
    const navItems = {
        "home": {
            label: "Home",
            isNavLink: true,
            exact: true,
            path: `/servers/${props.guild.guild.id}/`,
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

    return <div className="guild-page">
        <Switch>
            <Route path={`/servers/${props.guild.guild.id}/`} exact>
                <SideNav items={navItems} activePage={"home"}></SideNav>
                {/* <GuildSideNav guild={guild} activePage="home" ></GuildSideNav> */}
                <div className="guild-wrapper page-wrapper">
                    <GuildHome guild={props.guild} />
                </div>
            </Route>
            <Route path={`/servers/${props.guild.guild.id}/scripts/:scriptId/edit`}>
                {/* <GuildSideNav guild={guild} activePage="settings" ></GuildSideNav> */}
                <EditScript guild={props.guild}></EditScript>
            </Route>
        </Switch>
    </div>

    // <Route path={`/servers/${props.guild.guild.id}/scripts`}>
    // <SideNav items={navItems} activePage={"scripts"}></SideNav>
    // {/* <GuildSideNav guild={guild} activePage="scripts" ></GuildSideNav> */}
    // <div className="guild-wrapper page-wrapper">
    //     <GuildScripts guild={props.guild} />
    // </div>
    // </Route>
    // <Route path={`/servers/${props.guild.guild.id}/settings`}>
    // <SideNav items={navItems} activePage={"settings"}></SideNav>
    // {/* <GuildSideNav guild={guild} activePage="settings" ></GuildSideNav> */}
    // <div className="guild-wrapper page-wrapper">
    //     <GuildSettings guild={props.guild} />
    // </div>
    // </Route>
}

function EditScript(props: { guild: BotGuild }) {
    let params: { scriptId: string } = useParams();
    console.log(params);

    return <EditScriptPage guild={props.guild} scriptId={parseInt(params.scriptId)}></EditScriptPage>
}


function GuildHome(props: { guild: BotGuild }) {
    return <>
        <Panel>
            <p>This is a reminder that this service is currently in a early beta state and everything you're seeing is in a unfinished state, especially when it comes to this website.</p>
        </Panel>
        <PremiumPanel guild={props.guild}></PremiumPanel>
        <GuildScripts guild={props.guild}></GuildScripts>
    </>
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


    return <Panel>
        <h3>Premium/Lite</h3>
        {slots === null ? <p>Failed loading slots</p>
            : slots === undefined ? <p>Loading...</p>
                : slots.length > 0 ? slots.map(v => <div className="guild-premium-slots" key={v.id}>{v.tier} by <code>{v.user_id}</code></div>)
                    : <p>This server is on the free plan</p>}
    </Panel>
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
        <Panel>
            <h2>Create a new script</h2>
            <div className="create-script">
                <input type="text" ref={createScriptInput}></input>
                <AsyncOpButton label="Create" onClick={createScript}></AsyncOpButton>
            </div>
            {createError ? <p>Error creating script: <code>{createError}</code></p> : null}
            {scriptCreated ? <Redirect to={`/servers/${props.guild.guild.id}/scripts/${scriptCreated.id}/edit`}></Redirect> : null}
        </Panel >
        <Panel>
            <h2>Guild scripts</h2>
            {scripts ?
                <div className="scripts">
                    {scripts.map(script => <div key={script.id} className="script-item">
                        <p>#{script.id}</p>
                        <p>{script.enabled ? <span className="status-good">Enabled</span> : <span className="status-bad">Disabled</span>}</p>
                        <AsyncOpButton className="danger" label="delete" onClick={() => delScript(script.id)}></AsyncOpButton>
                        {script.enabled ?
                            <AsyncOpButton label="disable" onClick={() => toggleScript(script.id, false)}></AsyncOpButton>
                            :
                            <AsyncOpButton label="enable" onClick={() => toggleScript(script.id, true)}></AsyncOpButton>
                        }
                        <Link to={`/servers/${props.guild.guild.id}/scripts/${script.id}/edit`} className="bl-button">Edit</Link>
                        <p><code>{script.name}.ts</code></p>
                    </div>)}
                </div> : scripts === null ? <p>Failed loading scripts</p>
                    : <p>Loading...</p>
            }
        </Panel>
    </>
}