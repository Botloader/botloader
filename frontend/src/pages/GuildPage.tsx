import { useEffect, useState } from "react";
import { BotGuild, isErrorResponse, Script } from "botloader-common";
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
    }

    return <>
        <h2>Guild scripts</h2>
        {scripts ?
            <div className="scripts">
                {scripts.map(script => <div key={script.id} className="script-item">
                    <p>#{script.id}</p>
                    <p><code>{script.name}.ts</code></p>
                    <p>{script.enabled ? "Enabled" : "Disabled"}</p>
                    <AsyncOpButton className="danger" label="delete" onClick={() => delScript(script.id)}></AsyncOpButton>
                </div>)}
            </div> :
            <p>Loading...</p>
        }
    </>
}