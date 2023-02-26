import { Outlet, RouteObject, useParams, useRoutes } from "react-router-dom"
import { CurrentGuildProvider, GuildsProvider } from "./components/GuildsProvider"
import { RequireLoggedInSession, SessionProvider } from "./components/Session"
import { TopNav } from "./components/TopNav"
import { ConfirmLoginPage } from "./pages/ConfirmLogin"
import { NewsPage } from "./pages/NewsPage"
import { PrivacyPolicyPage } from "./pages/PrivacyPolicy"
import { SamplesPage } from "./pages/SamplesPage"
import { TosPage } from "./pages/TOS"
import { SelectServerPage } from './pages/SelectServer';
import { EditGuildScript, GuildHome, GuildPagesWrapper, GuildSideNav } from "./pages/GuildPage"
import { LandingPage } from "./pages/Landing"
import { SideNav } from "./components/SideNav"
import { UserGeneralPage } from "./pages/User/General"
import { UserPremiumPage } from "./pages/User/Premium"
import { UserScriptsPage } from "./pages/User/Scripts"
import { Box } from "@mui/material"
import { EditPluginPage } from "./pages/User/EditPlugin"
import { EditPluginScriptPage } from "./pages/User/EditPluginScript"
import { PluginProvider } from "./components/PluginProvider"

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
                    <SamplesPage />
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
                path: "/servers",
                element: <>
                    <TopNav />
                    <SelectServerPage />
                </>
            },
            {
                path: "user",
                element: <UserPages />,
                children: [
                    {
                        path: "general",
                        element: <UserGeneralPage />
                    },
                    {
                        path: "premium",
                        element: <UserPremiumPage />
                    },
                    {
                        path: "plugins",
                        element: <UserScriptsPage />,
                    },
                    {
                        path: "plugins/:pluginId",
                        element: <PluginProvider><EditPluginPage /></PluginProvider>
                    },
                    {
                        path: "plugins/:pluginId/edit_script",
                        element: <PluginProvider><EditPluginScriptPage initialDiff={false} /></PluginProvider>
                    },
                    {
                        path: "plugins/:pluginId/edit_script_diff",
                        element: <PluginProvider><EditPluginScriptPage initialDiff={true} /></PluginProvider>
                    }
                ]
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
                            <Box sx={{ display: 'flex' }}>

                                <GuildSideNav />
                                <Box
                                    component="main"
                                    sx={{ flexGrow: 1, bgcolor: 'background.default', p: 3 }}
                                >

                                    <GuildHome />
                                </Box>
                            </Box>
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

export function UserSideNav() {
    const navItems = {
        "General": {
            label: "General",
            isNavLink: true,
            exact: true,
            path: `/user/general`,
        },
        "Premium": {
            label: "Premium",
            isNavLink: true,
            exact: true,
            path: `/user/premium`,
        },
        "Plugins": {
            label: "Plugins",
            isNavLink: true,
            path: `/user/plugins`,
        },
    }

    return <SideNav items={navItems}></SideNav>
}

function UserPages() {
    return <>
        <RequireLoggedInSession>
            <TopNav />
            <Box sx={{ display: 'flex', flexGrow: 1 }}>
                <UserSideNav />
                <Box
                    component="main"
                    sx={{ display: "flex", flexGrow: 1, alignItems: "stretch", flexDirection: "column", bgcolor: 'background.default' }}
                >
                    <Outlet />
                </Box>
            </Box>
        </RequireLoggedInSession>
    </>
}
