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
        <p className="brand">BotLoader</p>
        <div className="top-nav-right">
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
    return <Link to="/settings">
        <img src={userAvatarUrl(props.user, 32)} alt="user avatar" className="avatar" />
        <p>{props.user.username}#{props.user.discriminator}</p>
    </Link>
}

function UserNotLoggedIn() {
    return <a href={BuildConfig.botloaderApiBase + "/login"}><p>Sign in</p></a>
}

function CurrentGuild(props: { guild: BotGuild }) {
    return <>
        <img src={guildIconUrl(props.guild.guild, 32)} alt="guild icon" className="avatar" />
        <p>{props.guild.guild.name}</p>
    </>
}

function NoCurrentGuild() {
    return <Link to="/servers">
        <p>Select server...</p>
    </Link>
}