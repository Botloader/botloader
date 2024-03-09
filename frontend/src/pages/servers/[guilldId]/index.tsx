import { GuildHome } from "./GuildPage";
import { Box } from "@mui/material";
import { routes as scriptRoutes } from "./scripts/[script_id]";
import { OurRouteObject } from "../../../misc/ourRoute";
import { GuildSideNav } from "../../../components/GuildSideNav";

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
        path: "scripts/:scriptId",
        children: scriptRoutes
    }
]