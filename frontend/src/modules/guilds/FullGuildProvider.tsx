import { FullGuild } from "botloader-common";
import {
    FetchData,
    FetchDataHookNotBehindGuard,
    createFetchDataContext,
    useFetchedData
} from "../../components/FetchData";
import { useSession } from "../session/useSession";
import { useCallback } from "react";

export const fullGuildContext = createFetchDataContext<FullGuild>();

type FullGuildHook = FetchDataHookNotBehindGuard<FullGuild>

export function FullGuildProvider({ children, guildId }: { guildId?: string, children: React.ReactNode }) {
    const session = useSession()

    const fetch = useCallback(async () => {
        if (!guildId) {
            throw new Error("guildId not set")
        }

        return await session.apiClient.getFullDiscordGuild(guildId);
    }, [session, guildId])

    if (!guildId || !session.user) {
        return <>{children}</>
    }

    return <FetchData
        context={fullGuildContext}
        loader={fetch}
        debugName="full_guild"
    >
        {children}
    </FetchData>
}

export function useCurrentFullGuild(): FullGuildHook {
    const fetchedData = useFetchedData(fullGuildContext)
    return fetchedData
}