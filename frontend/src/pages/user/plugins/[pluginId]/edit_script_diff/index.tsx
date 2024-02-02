import { RouteObject } from "react-router-dom";
import { EditPluginScriptPage } from "../EditPluginScript";

export const routes: RouteObject[] = [
    {
        index: true,
        handle: {
            breadCrumb: () => "Edit"
        },
        element: <EditPluginScriptPage initialDiff={true} />
    },
]