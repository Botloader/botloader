import { Outlet, useParams } from "react-router-dom"
import { routes as serverRoutes } from "./servers"
import { routes as pluginRoutes } from "./plugins"
import { routes as userRoutes } from "./user"
import { routes as completeStripePurchaseRoutes } from "./confirm_stripe_purchase"
import { routes as confirmLoginRoutes } from "./confirm_login"
import { TosPage } from "./TOS"
import { PrivacyPolicyPage } from "./PrivacyPolicy"
import { TopNav } from "../components/TopNav"
import { LandingPage } from "./Landing"
import { NewsPage } from "./NewsPage"
import { SamplesPage } from "./SamplesPage"
import { BotloaderWebsocketProvider } from "../modules/websocket/WebsocketContext"
import { CurrentGuildProvider } from "../modules/guilds/CurrentGuild"
import { GuildScriptsProvider } from "../modules/guilds/GuildScriptsProvider"
import { OurRouteObject } from "../misc/ourRoute"
import { MaybePluginProvider } from "../components/PluginProvider"

export const routes: OurRouteObject[] = [
    {
        path: "/tos",
        element: <TosPage />,
    },
    {
        path: "/privacy",
        element: <PrivacyPolicyPage />
    },
    {
        element: <MainRoutesElement />,
        children: [
            {
                index: true,
                handle: {
                    breadCrumb: () => "Home"
                },
                element: <>
                    <LandingPage />
                </>
            },
            {
                path: "news",
                handle: {
                    breadCrumb: () => "News"
                },
                element: <>
                    <NewsPage />
                </>,
            },
            {
                path: "samples",
                element: <>
                    <SamplesPage />
                </>
            },
            {
                path: "premium",
                element: <>
                    TODO
                </>
            },
            {
                path: "/servers",
                children: serverRoutes
            },
            {
                path: "/user",
                children: userRoutes
            },
            {
                path: "/plugins",
                handle: {
                    breadCrumb: () => "Plugins"
                },
                children: pluginRoutes
            },
            {
                path: "/confirm_login",
                children: confirmLoginRoutes
            },
            {
                path: "/confirm_stripe_purchase",
                children: completeStripePurchaseRoutes
            },
        ]
    }
]

function MainRoutesElement() {
    let params = useParams();

    return <CurrentGuildProvider guildId={params.guildId}>
        <BotloaderWebsocketProvider>
            <GuildScriptsProvider guildId={params.guildId}>
                <MaybePluginProvider pluginId={params.pluginId ? parseInt(params.pluginId) : undefined}>
                    <TopNav />
                    <Outlet></Outlet>
                </MaybePluginProvider>
            </GuildScriptsProvider>
        </BotloaderWebsocketProvider>
    </CurrentGuildProvider>

}