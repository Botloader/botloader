import { UserScriptsPage } from "./Scripts";
import { routes as pluginIdRoutes } from "./[pluginId]";
import { OurRouteObject } from "../../../misc/ourRoute";

export const routes: OurRouteObject[] = [
    {
        handle: {
            breadCrumb: () => "Plugins"
        },
        children: [
            {
                index: true,
                element: <UserScriptsPage />
            },
            {
                path: ":pluginId",
                children: pluginIdRoutes,
            }
        ]
    }
]