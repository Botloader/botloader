import { useEffect, useState } from "react";
import { ApiClient, BotGuild, isErrorResponse, Script } from "botloader-common";
import { useCurrentGuild } from "../components/GuildsProvider";
import { useSession } from "../components/Session";
import './GuildPage.css'
import { AsyncOpButton } from "../components/AsyncOpButton";
import { BuildConfig } from "../BuildConfig";

export function GuildPage() {
    let guild = useCurrentGuild();
    if (guild) {
        if (guild.connected) {
            return <GuildControlPage guild={guild} />
        } else {
            return <InviteGuildPage guild={guild} />
        }
    } else {
        return <NoGuildPage />
    }
}

function InviteGuildPage(props: { guild: BotGuild }) {
    return <a href={`https://discord.com/api/oauth2/authorize?client_id=${BuildConfig.botloaderClientId}&permissions=532844244928&scope=bot%20applications.commands&guild_id=${props.guild.guild.id}`} className="add-to-server">Click here to add to server!</a>;
}

function NoGuildPage() {
    return <p>That's and unknown guild m8</p>
}

function GuildControlPage(props: { guild: BotGuild }) {
    const [scripts, setScripts] = useState<Script[] | undefined>(undefined);
    const session = useSession();

    async function loadScripts() {
        let resp = await session.apiClient.getAllScripts(props.guild.guild.id);
        if (isErrorResponse(resp)) {
            // TODO
            setScripts(undefined);
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

    return <>
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
            </div> :
            <p>Loading...</p>
        }
    </>
}