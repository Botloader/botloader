import { ApiResult, Script, ScriptsWithPlugins, isErrorResponse } from "botloader-common";
import { FetchData, FetchDataHookNotBehindGuard, createFetchDataContext, useFetchedData } from "../../components/FetchData";
import { useSession } from "../session/useSession";
import { useParams } from "react-router-dom";
import { useCallback } from "react";

export const guildScriptsContext = createFetchDataContext<ScriptsWithPlugins>();

type GuildScriptsHook = {
    delScript: (scriptId: number) => void,
    toggleScript: (scriptId: number, enabled: boolean) => void,
    createScript: (name: string) => Promise<ApiResult<Script>>,
} & FetchDataHookNotBehindGuard<ScriptsWithPlugins>

export function GuildScriptsProvider({ children, guildId }: { guildId?: string, children: React.ReactNode }) {
    const session = useSession()

    const fetch = useCallback(async () => {
        if (!guildId) {
            throw new Error("guildId not set")
        }

        return await session.apiClient.getAllScriptsWithPlugins(guildId);
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
        let resp = await session.apiClient.delScript(safeGuildId, scriptId);
        if (!isErrorResponse(resp)) {
            fetchedData.reload();
        }

        await session.apiClient.reloadGuildVm(safeGuildId);
    }

    async function toggleScript(scriptId: number, enabled: boolean) {
        let resp = await session.apiClient.updateScript(safeGuildId, scriptId, {
            enabled,
        });
        if (!isErrorResponse(resp)) {
            await session.apiClient.reloadGuildVm(safeGuildId);
            fetchedData.reload();
        }
    }

    async function createScript(name: string) {
        let resp = await session.apiClient.createScript(safeGuildId, {
            enabled: false,
            name: name,
            original_source: "",
        })

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