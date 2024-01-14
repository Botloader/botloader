import { RouteObject } from "react-router-dom";
import { GuildHome, GuildSideNav } from "./GuildPage";
import { Box } from "@mui/material";
import { routes as editScriptRoutes } from "./scripts/[script_id]/edit";

export const routes: RouteObject[] = [
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
        </>,
    },
    {
        path: "scripts/:scriptId/edit",
        children: editScriptRoutes
    }
]