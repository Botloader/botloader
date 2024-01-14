import { RouteObject } from "react-router-dom";
import { routes as pluginIdRoutes } from "./[pluginId]";
import { TopNav } from "../../components/TopNav";
import { ViewPlugins } from "./Plugins";

export const routes: RouteObject[] = [
    {
        children: [
            {
                index: true,
                element: <>
                    <TopNav />
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