import { Outlet, RouteObject, useParams } from "react-router-dom";
import { TopNav } from "../../components/TopNav";
import { SelectServerPage } from "./SelectServer";
import { routes as serverIdRoutes } from "./[id]";
import { RequireLoggedInSession } from "../../components/Session";
import { CurrentGuildProvider, useCurrentGuild } from "../../components/GuildsProvider";
import { BotGuild } from "botloader-common";
import { BuildConfig } from "../../BuildConfig";

export const routes: RouteObject[] = [
    {
        index: true,
        element: <>
            <TopNav />
            <SelectServerPage />
        </>
    },
    {
        path: ":guildId",
        children: serverIdRoutes,
        element: <>
            <RequireLoggedInSession>
                <GuildPages />
            </RequireLoggedInSession>
        </>,
    }
]

function GuildPages() {
    let { guildId } = useParams();

    return <CurrentGuildProvider guildId={guildId}>
        <TopNav />
        <GuildPagesWrapper>
            <Outlet />
        </GuildPagesWrapper>
    </CurrentGuildProvider>
}



export function GuildPagesWrapper({ children }: { children: React.ReactNode }) {
    let guild = useCurrentGuild();
    if (guild) {
        if (guild.connected) {
            return <>
                {children}
            </>
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