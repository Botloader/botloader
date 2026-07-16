import { ApiResult, Script, GetScriptsWithPluginsResponse, isErrorResponse } from "botloader-common";
import { FetchData, FetchDataHookNotBehindGuard, createFetchDataContext, useFetchedData } from "../../components/FetchData";
import { useSession } from "../session/useSession";
import { useParams } from "react-router-dom";
import { useCallback } from "react";

export const guildScriptsContext = createFetchDataContext<GetScriptsWithPluginsResponse>();

type GuildScriptsHook = {
    delScript: (scriptId: number) => Promise<void>,
    toggleScript: (scriptId: number, enabled: boolean) => Promise<void>,
    createScript: (name: string) => Promise<ApiResult<Script>>,
} & FetchDataHookNotBehindGuard<GetScriptsWithPluginsResponse>

export function GuildScriptsProvider({ children, guildId }: { guildId?: string, children: React.ReactNode }) {
    const session = useSession()

    const fetch = useCallback(async () => {
        if (!guildId) {
            throw new Error("guildId not set")
        }

        return await session.apiClient.operations.get_all_guild_scripts_with_plugins({ guild: guildId });
    }, [session, guildId])

    if (!guildId || !session.user) {
        return <>{children}</>
    }

    return <FetchData
        context={guildScriptsContext}
        loader={fetch}
        debugName="guild_scripts"
    >
        {children}
    </FetchData>
}

export function useCurrentGuildScripts(): GuildScriptsHook {
    const fetchedData = useFetchedData(guildScriptsContext)
    const session = useSession()
    const { guildId } = useParams()

    if (!guildId) {
        throw new Error("No guild id param")
    }
    const safeGuildId = guildId

    async function delScript(scriptId: number) {
        let resp = await session.apiClient.operations.delete_guild_script({ guild: safeGuildId, script_id: scriptId });
        if (!isErrorResponse(resp)) {
            fetchedData.reload();
        }

        await session.apiClient.operations.reload_guild_vm({ guild: safeGuildId });
    }

    async function toggleScript(scriptId: number, enabled: boolean) {
        let resp = await session.apiClient.operations.update_guild_script({ guild: safeGuildId, script_id: scriptId, data: {
            enabled,
        } });
        if (!isErrorResponse(resp)) {
            fetchedData.reload();
        }
    }

    async function createScript(name: string) {
        let resp = await session.apiClient.operations.create_guild_script({ guild: safeGuildId, data: {
            enabled: false,
            name: name,
            original_source: "",
        } })

        if (!isErrorResponse(resp)) {
            fetchedData.reload()
        }

        return resp
    }

    return {
        ...fetchedData,
        delScript,
        toggleScript,
        createScript,
    }
}