import { useMemo } from "react";
import { Link } from "react-router-dom";
import { BotGuild, UserGuild } from "botloader-common";
import { useGuilds } from "../components/GuildsProvider"
import { guildIconUrl } from "../components/Util";
import "./SelectServer.css"
import { Panel } from "../components/Panel";

export function SelectServerPage() {

    const guilds = useGuilds();

    const [joinedHasAdmin, notJoinedHasAdmin] = useMemo(() => {
        if (!guilds) {
            return [[], []];
        }

        const guildsAdmins = guilds.guilds.filter(g => hasAdmin(g.guild));
        const joinedHasAdmin = guildsAdmins.filter(g => g.connected);
        const notJoinedHasAdmin = guildsAdmins.filter(g => !g.connected);

        return [joinedHasAdmin, notJoinedHasAdmin];

    }, [guilds])

    if (!guilds) {
        return <p>Loading guilds.... (unless you're not logged in that is)</p>
    }

    return <div className="guild-select-page">
        <Panel>
            <h2>Alpha version</h2>
            <p>Botloader is currently in a alpha state, everything you're seeing is work in progress and will likely change a lot</p>
            <p><a href="/docs/">Scripting API Documentation</a></p>
        </Panel>
        <Panel>
            <h2>Joined servers</h2>
            <div className="guild-select-list">
                {joinedHasAdmin.map(g => <GuildListItem guild={g} key={g.guild.id} />)}
            </div >
        </Panel>

        <Panel>
            <h2>Add to new servers</h2>
            <div className="guild-select-list">
                {notJoinedHasAdmin.map(g => <GuildListItem guild={g} key={g.guild.id} />)}
            </div >
        </Panel>
    </div>
}

function GuildListItem({ guild: g }: { guild: BotGuild }) {
    return <Link to={`/servers/${g.guild.id}`}><div className="guild-list-item">
        <GuildIcon guild={g.guild} />
        <p>{shorten(g.guild.name)}</p>
    </div></Link>
}

function GuildIcon(props: { guild: UserGuild }) {
    return <img src={guildIconUrl(props.guild)} alt={`?`} className="avatar" />
}

const permAdmin = BigInt("0x0000000008");
const permManageServer = BigInt("0x0000000020");

function hasAdmin(g: UserGuild): boolean {
    if (g.owner) {
        return true
    }


    const perms = BigInt(g.permissions);
    if ((perms & permAdmin) === permAdmin) {
        return true
    }

    if ((perms & permManageServer) === permManageServer) {
        return true
    }

    return false
}

function shorten(name: string): string {
    const maxLength = 35
    return name.length > maxLength ? name.slice(0, 34) + '...' : name
}
