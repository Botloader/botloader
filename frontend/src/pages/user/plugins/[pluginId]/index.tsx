import { Outlet, RouteObject } from "react-router-dom";
import { EditPluginPage } from "./EditPlugin";
import { PluginProvider } from "../../../../components/PluginProvider";
import { routes as editScriptRoutes } from "./edit_script";
import { routes as editScriptDiffRoutes } from "./edit_script_diff";

export const routes: RouteObject[] = [
    {
        element: <PluginProvider><Outlet /></PluginProvider>,
        children: [
            {
                index: true,
                element: <EditPluginPage />
            },
            {
                path: "edit_script",
                children: editScriptRoutes
            },
            {
                path: "edit_script_diff",
                children: editScriptDiffRoutes
            }
        ]
    },
]