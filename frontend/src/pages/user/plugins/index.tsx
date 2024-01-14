import { RouteObject } from "react-router-dom";
import { UserScriptsPage } from "./Scripts";
import { routes as pluginIdRoutes } from "./[pluginId]";

export const routes: RouteObject[] = [
    {
        index: true,
        element: <UserScriptsPage />
    },
    {
        path: ":pluginId",
        children: pluginIdRoutes,
    }
]