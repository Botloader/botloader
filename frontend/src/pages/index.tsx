import { Outlet, RouteObject } from "react-router-dom"
import { routes as serverRoutes } from "./servers"
import { routes as pluginRoutes } from "./plugins"
import { routes as userRoutes } from "./user"
import { routes as confirmLoginRoutes } from "./confirm_login"
import { TosPage } from "./TOS"
import { PrivacyPolicyPage } from "./PrivacyPolicy"
import { TopNav } from "../components/TopNav"
import { LandingPage } from "./Landing"
import { NewsPage } from "./NewsPage"
import { SamplesPage } from "./SamplesPage"
import { SessionProvider } from "../components/Session"
import { GuildsProvider } from "../components/GuildsProvider"

export const routes: RouteObject[] = [
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
                children: serverRoutes
            },
            {
                path: "/user",
                children: userRoutes
            },
            {
                path: "/plugins",
                children: pluginRoutes
            },
            {
                path: "/confirm_login",
                children: confirmLoginRoutes
            },
        ]
    }
]

function MainRoutesElement() {
    return <SessionProvider>
        <GuildsProvider>
            <Outlet></Outlet>
        </GuildsProvider>
    </SessionProvider>
}