import { Outlet } from "react-router-dom";
import { pluginContext } from "../../../components/PluginProvider";
import { ViewPlugin, ViewPluginSource } from "./ViewPlugin";
import { FetchDataGuard } from "../../../components/FetchData";
import { OurRouteObject } from "../../../misc/ourRoute";

export const routes: OurRouteObject[] = [
    {
        element: <>
            <FetchDataGuard context={pluginContext}>
                <Outlet />
            </FetchDataGuard>
        </>,
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
                element: <ViewPlugin />
            },
            {
                path: "source",
                element: <ViewPluginSource />
            }
        ]
    }
]
