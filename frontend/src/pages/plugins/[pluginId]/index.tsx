import { Outlet, RouteObject } from "react-router-dom";
import { PluginProvider } from "../../../components/PluginProvider";
import { TopNav } from "../../../components/TopNav";
import { ViewPlugin, ViewPluginSource } from "./ViewPlugin";

export const routes: RouteObject[] = [
    {
        element: <>
            <TopNav />
            <PluginProvider>
                <Outlet />
            </PluginProvider>
        </>,
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
