import { Link } from "react-router-dom";
import { BotGuild, User } from "botloader-common";
import { BuildConfig } from "../BuildConfig";
import { useCurrentGuild } from "./GuildsProvider";
import { useSession } from "./Session";
import "./TopNav.css"
import { guildIconUrl, userAvatarUrl } from "./Util";

export function TopNav() {
    let session = useSession();
    let currentGuild = useCurrentGuild();

    return <div className="top-nav">
        <p className="brand"><Link to="/">Botloader <small>beta</small></Link></p>
        <div className="top-nav-right">
            <a href="/docs/" target="_blank">Docs</a>
            <a href="/book/" target="_blank">Guide</a>
            <Link to="/news">News</Link>
            <Link to="/samples">Samples</Link>
            {/* <Link to="/premium">Premium</Link> */}
            <div className="current-server">
                {currentGuild && session.user ? <CurrentGuild guild={currentGuild} /> : session.user ? <NoCurrentGuild /> : null}
            </div>
            <div className="current-user">
                {session.user ? <UserLoggedIn user={session.user} /> : <UserNotLoggedIn />}
            </div>
        </div>
    </div>
}

function UserLoggedIn(props: { user: User }) {
    return <Link to="/settings" className="bl-button">
        <img src={userAvatarUrl(props.user, 32)} alt="user avatar" className="avatar" />
        <p>{props.user.username}#{props.user.discriminator}</p>
    </Link>
}

function UserNotLoggedIn() {
    return <a href={BuildConfig.botloaderApiBase + "/login"}><p>Sign in</p></a>
}

function CurrentGuild(props: { guild: BotGuild }) {
    return <Link to="/servers">
        {props.guild.guild.icon ? <img src={guildIconUrl(props.guild.guild, 32)} alt="guild icon" className="avatar" /> : null}
        <p>{props.guild.guild.name}</p>
    </Link>
}

function NoCurrentGuild() {
    return <Link to="/servers">
        Server Selection
    </Link>
}