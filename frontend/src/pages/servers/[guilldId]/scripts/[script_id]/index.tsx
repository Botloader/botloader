import { OurRouteObject } from "../../../../../misc/ourRoute";
import { routes as editScriptRoutes } from "./edit";
import { Box } from "@mui/material";
import { GuildSideNav } from "../../../../../components/GuildSideNav";
import { GuildScriptPage } from "../../../../../components/GuildScriptPage";

export const routes: OurRouteObject[] = [{
    // index: true,
    handle: {
        breadCrumb: (params, data) => {
            if (params.scriptId && data.currentGuildScripts?.scripts) {
                const script = data.currentGuildScripts.scripts.find(v => v.id + "" === params.scriptId)
                if (script) {
                    return script.name
                }
            }

            return params.scriptId ?? "unknown script"
        },
    },
    children: [{
        index: true,
        element: <>
            <Box sx={{ display: 'flex' }}>
                <GuildSideNav />
                <Box
                    component="main"
                    sx={{ flexGrow: 1, bgcolor: 'background.default', p: 3 }}
                >
                    <GuildScriptPage />
                </Box>
            </Box>
        </>,
    }, {
        path: "edit",
        children: editScriptRoutes
    }]
}]