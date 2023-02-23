import { Plugin } from "botloader-common";
import { useParams } from "react-router-dom";
import { createFetchDataContext, FetchDataGuarded } from "./FetchData";
import { useSession } from "./Session";

export const pluginContext = createFetchDataContext<Plugin>();


export function PluginProvider({ children }: { children: React.ReactNode }) {
    const { pluginId } = useParams();
    const session = useSession();

    async function fetchPlugin() {
        let scripts = await session.apiClient.getPlugin(parseInt(pluginId!));
        return scripts;
    }

    return <FetchDataGuarded loader={fetchPlugin} context={pluginContext}>
        {children}
    </FetchDataGuarded>
}