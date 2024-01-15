import { Outlet, RouteObject } from "react-router-dom";
import { routes as generalRoutes } from "./general";
import { routes as pluginRoutes } from "./plugins";
import { routes as premiumRoutes } from "./premium";
import { RequireLoggedInSession } from "../../components/Session";
import { TopNav } from "../../components/TopNav";
import { Box } from "@mui/material";
import { SideNav } from "../../components/SideNav";

export const routes: RouteObject[] = [
    {
        element: <UserPages />,
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