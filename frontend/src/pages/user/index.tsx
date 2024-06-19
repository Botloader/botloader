import { Outlet, RouteObject } from "react-router-dom";
import { routes as generalRoutes } from "./general";
import { routes as pluginRoutes } from "./plugins";
import { routes as premiumRoutes } from "./premium";
import { Box } from "@mui/material";
import { SideNav } from "../../components/SideNav";
import { RequireLoggedInSession } from "../../modules/session/RequireLoggedInSession";
import { createContext, useState } from "react";

export const routes: RouteObject[] = [
    {
        element: <UserPages />,
        handle: {
            breadCrumb: () => "User",
            breadCrumbCosmeticOnly: true,
        },
        children: [
            {
                path: "general",
                children: generalRoutes,
            },
            {
                path: "premium",
                children: premiumRoutes,
            },
            {
                path: "plugins",
                children: pluginRoutes,
            },
        ]
    }
]

export function UserSideNav() {
    const navItems = [
        {
            label: "General",
            isNavLink: true,
            exact: true,
            path: `/user/general`,
        },
        {
            label: "Premium",
            isNavLink: true,
            exact: true,
            path: `/user/premium`,
        },
        {
            label: "Plugins",
            isNavLink: true,
            path: `/user/plugins`,
        },
    ]

    return <SideNav items={navItems}></SideNav>
}

export const UserSideNavContext = createContext({
    setShown: (shown: boolean) => { }
})

function UserPages() {
    const [sideNavShown, setSideNavShown] = useState(true)

    return <>
        <RequireLoggedInSession>
            <Box sx={{ display: 'flex', flexGrow: 1 }}>
                {sideNavShown && <UserSideNav />}
                <Box
                    component="main"
                    sx={{ display: "flex", flexGrow: 1, alignItems: "stretch", flexDirection: "column", bgcolor: 'background.default' }}
                >
                    <UserSideNavContext.Provider value={{ setShown: (value) => setSideNavShown(value) }}>
                        <Outlet />
                    </UserSideNavContext.Provider>
                </Box>
            </Box>
        </RequireLoggedInSession>
    </>
}