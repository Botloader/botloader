import { RouteObject } from "react-router-dom";
import { routes as pluginIdRoutes } from "./[pluginId]";
import { ViewPlugins } from "./Plugins";

export const routes: RouteObject[] = [
    {
        children: [
            {
                index: true,
                element: <>
                    <ViewPlugins />
                </>
            },
            {
                path: ":pluginId",
                children: pluginIdRoutes,
            }
        ]
    }
]