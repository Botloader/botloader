import { Outlet, RouteObject, useParams, useRoutes } from "react-router-dom"
import { CurrentGuildProvider, GuildsProvider } from "./components/GuildsProvider"
import { RequireLoggedInSession, SessionProvider } from "./components/Session"
import { TopNav } from "./components/TopNav"
import { ConfirmLoginPage } from "./pages/ConfirmLogin"
import { NewsPage } from "./pages/NewsPage"
import { PrivacyPolicyPage } from "./pages/PrivacyPolicy"
import { SamplesPage } from "./pages/SamplesPage"
import { TosPage } from "./pages/TOS"
import { UserSettingsPage } from "./pages/UserSettings"
import { SelectServerPage } from './pages/SelectServer';
import { EditGuildScript, GuildHome, GuildPagesWrapper, GuildSideNav } from "./pages/GuildPage"
import { LandingPage } from "./pages/Landing"

export function RoutesElement() {
    let routes = useRoutes(appRoutes);
    return routes;
}

const appRoutes: RouteObject[] = [
    {
        path: "/confirm_login",
        element: <ConfirmLoginPage />
    },
    {
        path: "/tos",
        element: <TosPage />,
    },
    {
        path: "/privacy",
        element: <PrivacyPolicyPage />
    },
    {
        path: "/",
        element: <MainRoutesElement />,
        children: [
            {
                index: true,
                element: <>
                    <TopNav />
                    <LandingPage />
                </>
            },
            {
                path: "news",
                element: <>
                    <TopNav />
                    <NewsPage />
                </>,
            },
            {
                path: "samples",
                element: <>
                    <TopNav />
                    <div className="page-wrapper"><SamplesPage /></div>
                </>
            },
            {
                path: "premium",
                element: <>
                    <TopNav />
                    TODO
                </>
            },
            {
                path: "settings",
                element: <>
                    <TopNav />
                    <RequireLoggedInSession>
                        <div className="page-wrapper"><UserSettingsPage /></div>
                    </RequireLoggedInSession>
                </>
            },
            {
                path: "/servers",
                element: <>
                    <TopNav />
                    <div className="page-wrapper"><SelectServerPage /></div>
                </>
            },
            {
                path: "/servers/:guildId",
                element: <>
                    <RequireLoggedInSession>
                        <GuildPages />
                    </RequireLoggedInSession>
                </>,
                children: [
                    {
                        index: true,
                        element: <>
                            <GuildSideNav />
                            {/* <GuildSideNav guild={guild} activePage="home" ></GuildSideNav> */}
                            <div className="guild-wrapper page-wrapper">
                                <GuildHome />
                            </div>
                        </>
                    },
                    {
                        path: "scripts/:scriptId/edit",
                        element: <EditGuildScript />
                    }
                ]
            }
        ],
    }
]

function MainRoutesElement() {
    return <SessionProvider>
        <GuildsProvider>
            <Outlet></Outlet>
        </GuildsProvider>
    </SessionProvider>
}

function GuildPages() {
    let { guildId } = useParams();

    return <CurrentGuildProvider guildId={guildId}>
        <TopNav />
        <GuildPagesWrapper>
            <Outlet />
        </GuildPagesWrapper>
    </CurrentGuildProvider>
}