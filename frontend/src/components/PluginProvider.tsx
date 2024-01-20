import { Plugin } from "botloader-common";
import { createFetchDataContext, FetchData } from "./FetchData";
import { useSession } from "../modules/session/useSession";
import { useCallback } from "react";

export const pluginContext = createFetchDataContext<Plugin>();


export function MaybePluginProvider({ pluginId, children }: { pluginId?: number, children: React.ReactNode }) {
    const session = useSession();

    const fetchPlugin = useCallback(async () => {
        if (!pluginId) {
            throw new Error("No plugin id")
        }

        let scripts = await session.apiClient.getPlugin(pluginId!);
        return scripts;
    }, [pluginId, session])

    return <FetchData loader={fetchPlugin} context={pluginContext}>
        {children}
    </FetchData>
}