import { GuildHome, GuildSideNav } from "./GuildPage";
import { Box } from "@mui/material";
import { routes as editScriptRoutes } from "./scripts/[script_id]/edit";
import { OurRouteObject } from "../../../misc/ourRoute";

export const routes: OurRouteObject[] = [
    {
        index: true,
        // handle: {
        //     breadCrumb: () => "Home"
        // },
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