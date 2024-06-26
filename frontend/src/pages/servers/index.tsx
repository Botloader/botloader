import { Outlet } from "react-router-dom";
import { SelectServerPage } from "./SelectServer";
import { routes as serverIdRoutes } from "./[guilldId]";
import { BotGuild } from "botloader-common";
import { BuildConfig } from "../../BuildConfig";
import { OurRouteObject } from "../../misc/ourRoute";
import { RequireLoggedInSession } from "../../modules/session/RequireLoggedInSession";
import { useCurrentGuild } from "../../modules/guilds/CurrentGuild";
import { Loading } from "../../components/Loading";
import { useEffect } from "react";

export const routes: OurRouteObject[] = [
    {
        handle: {
            breadCrumb: () => "Servers"
        },
        children: [
            {
                index: true,
                element: <>
                    <SelectServerPage />
                </>
            },
            {
                path: ":guildId",
                children: serverIdRoutes,
                handle: {
                    breadCrumb: ({ guildId }, { userGuilds }) => {
                        let guild = userGuilds?.find(v => v.guild.id === guildId)
                        if (guild) {
                            return guild.guild.name
                        } else {
                            return guildId ?? ""
                        }
                    }
                },
                element: <>
                    <RequireLoggedInSession>
                        <GuildPages />
                    </RequireLoggedInSession>
                </>,
            }]
    }
]

function GuildPages() {
    return <>
        <GuildPagesWrapper>
            <Outlet />
        </GuildPagesWrapper>
    </>
}



export function GuildPagesWrapper({ children }: { children: React.ReactNode }) {
    let guild = useCurrentGuild();
    if (guild?.loading || guild?.waiting) {
        return <Loading />
    }

    if (guild?.value) {
        if (guild.value.connected) {
            return <>
                {children}
            </>
        } else {
            return <div className="page-wrapper">
                <InviteGuildPage guild={guild.value} />
            </div>
        }
    } else {
        return <div className="page-wrapper">
            <NoGuildPage />
        </div>
    }
}


function InviteGuildPage(props: { guild: BotGuild }) {
    const baseUrl = window.location.origin
    const redirectUrl = encodeURIComponent(baseUrl + "/servers/")
    const addBotUrl = `https://discord.com/api/oauth2/authorize?client_id=${BuildConfig.botloaderClientId}&permissions=515463572672&scope=bot%20applications.commands&guild_id=${props.guild.guild.id}&redirect_uri=${redirectUrl}&response_type=code`
    useEffect(() => {
        window.location.href = addBotUrl
    }, [addBotUrl])

    return <a href={`https://discord.com/api/oauth2/authorize?client_id=${BuildConfig.botloaderClientId}&permissions=515463572672&scope=bot%20applications.commands&guild_id=${props.guild.guild.id}`} className="add-to-server" target="_blank" rel="noreferrer">Click here to add to server!</a>;
}

function NoGuildPage() {
    return <p>That's and unknown guild m8</p>
}