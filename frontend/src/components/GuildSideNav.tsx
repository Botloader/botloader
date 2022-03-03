// import { Link } from "react-router-dom";
// import { BotGuild, User } from "botloader-common";
// import { BuildConfig } from "../BuildConfig";
import { BotGuild } from "botloader-common";
import { NavLink } from "react-router-dom";
// import { useSession } from "./Session";
import "./GuildSideNav.css"
// import { guildIconUrl, userAvatarUrl } from "./Util";

export function GuildSideNav(props: { guild: BotGuild }) {

    return <nav className="guild-nav">
        <ul>
            <li><NavLink to={`/servers/${props.guild.guild.id}/`} exact={true}>Home</NavLink></li>
            <li><NavLink to={`/servers/${props.guild.guild.id}/scripts`}>Scripts</NavLink></li>
            <li><NavLink to={`/servers/${props.guild.guild.id}/settings`}>Settings</NavLink></li>
        </ul>
    </nav >
}
