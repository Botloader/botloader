import { useEffect, useState } from "react";
import { BotGuild, GuildMetaConfig, isErrorResponse, Script } from "botloader-common";
import { useCurrentGuild } from "../components/GuildsProvider";
import { useSession } from "../components/Session";
import './GuildPage.css'
import { AsyncOpButton } from "../components/AsyncOpButton";
import { BuildConfig } from "../BuildConfig";
import { Route, Switch } from "react-router-dom";
import { Panel } from "../components/Panel";
import { SideNav, SideNavItemMap } from "../components/SideNav";

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
    return <a href={`https://discord.com/api/oauth2/authorize?client_id=${BuildConfig.botloaderClientId}&permissions=532844244928&scope=bot%20applications.commands&guild_id=${props.guild.guild.id}`} className="add-to-server" target="_blank" rel="noreferrer">Click here to add to server!</a>;
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
        "scripts": {
            label: "Scripts",
            isNavLink: true,
            exact: true,
            path: `/servers/${props.guild.guild.id}/scripts`,
        },
        "settings": {
            label: "Settings",
            isNavLink: true,
            exact: true,
            path: `/servers/${props.guild.guild.id}/settings`,
        },
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
            <Route path={`/servers/${props.guild.guild.id}/scripts`}>
                <SideNav items={navItems} activePage={"scripts"}></SideNav>
                {/* <GuildSideNav guild={guild} activePage="scripts" ></GuildSideNav> */}
                <div className="guild-wrapper page-wrapper">
                    <GuildScripts guild={props.guild} />
                </div>
            </Route>
            <Route path={`/servers/${props.guild.guild.id}/settings`}>
                <SideNav items={navItems} activePage={"settings"}></SideNav>
                {/* <GuildSideNav guild={guild} activePage="settings" ></GuildSideNav> */}
                <div className="guild-wrapper page-wrapper">
                    <GuildSettings guild={props.guild} />
                </div>
            </Route>
        </Switch>
    </div>
}


function GuildHome(props: { guild: BotGuild }) {
    return <Panel>
        <p>Eventually we will display a bunch of usefull stuff here</p>
        <p>This is a reminder that this service is currently in a ALPHA state and everything you're seeing is in a unfinished state.</p>
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

    return <Panel>
        <h2>Guild scripts</h2>
        {scripts ?
            <div className="scripts">
                {scripts.map(script => <div key={script.id} className="script-item">
                    <p>#{script.id}</p>
                    <p><code>{script.name}.ts</code></p>
                    <p>{script.enabled ? <span className="status-good">Enabled</span> : <span className="status-bad">Disabled</span>}</p>
                    <AsyncOpButton className="danger" label="delete" onClick={() => delScript(script.id)}></AsyncOpButton>
                    {script.enabled ?
                        <AsyncOpButton className="primary" label="disable" onClick={() => toggleScript(script.id, false)}></AsyncOpButton>
                        :
                        <AsyncOpButton className="primary" label="enable" onClick={() => toggleScript(script.id, true)}></AsyncOpButton>
                    }
                </div>)}
            </div> : scripts === null ? <p>Failed loading scripts</p>
                : <p>Loading...</p>
        }
    </Panel>
}