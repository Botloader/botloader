import { RouteObject } from "react-router-dom";
import { EditPluginScriptPage } from "../../../EditPluginScript";

export const routes: RouteObject[] = [
    {
        index: true,
        element: <EditPluginScriptPage initialDiff={false} />
    },
]