import { Outlet } from "react-router-dom";
import { EditPluginPage } from "./EditPlugin";
import { pluginContext } from "../../../../components/PluginProvider";
import { routes as editScriptRoutes } from "./edit_script";
import { routes as editScriptDiffRoutes } from "./edit_script_diff";
import { FetchDataGuard } from "../../../../components/FetchData";
import { OurRouteObject } from "../../../../misc/ourRoute";

export const routes: OurRouteObject[] = [
    {
        element: <FetchDataGuard context={pluginContext}><Outlet /></FetchDataGuard>,
        handle: {
            breadCrumb: ({ pluginId }, { currentPlugin }) => {
                if (currentPlugin?.id + "" === pluginId) {
                    return currentPlugin?.name ?? ""
                }

                return pluginId ?? "unknown plugin"
            }
        },
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