import { BotGuild, isErrorResponse, UserGuild } from "botloader-common";
import { createFetchDataContext, FetchData, FetchDataGuard, useFetchedData } from "../../components/FetchData";
import { useSession } from "../session/useSession";
import { useCallback } from "react";

export const GuildsContext = createFetchDataContext<LoadedGuilds>();

export function GuildsProvider({ children }: { children: React.ReactNode }) {
    const session = useSession();

    const fetch = useCallback(async () => {
        if (!session.user) {
            return undefined
        }

        const resp = await session.apiClient.getCurrentUserGuilds();
        if (isErrorResponse(resp)) {
            return resp
        }

        const adminGuilds = resp.guilds.filter((g) => userGuildHasAdmin(g.guild));

        return {
            all: resp.guilds,
            hasAdmin: adminGuilds,
        }
    }, [session])

    return <FetchData loader={fetch} context={GuildsContext} debugName="guilds">
        {children}
    </FetchData>
}

export function GuildsGuard({ children }: { children: React.ReactNode }) {
    return <FetchDataGuard context={GuildsContext}>{children}</FetchDataGuard>
}

export function useGuilds() {
    return useFetchedData(GuildsContext)
}

interface LoadedGuilds {
    all: BotGuild[],
    hasAdmin: BotGuild[],
}

const permAdmin = BigInt("0x0000000008");
const permManageServer = BigInt("0x0000000020");

function userGuildHasAdmin(g: UserGuild): boolean {
    if (g.owner) {
        return true
    }


    const perms = BigInt(g.permissions);
    if ((perms & permAdmin) === permAdmin) {
        return true
    }

    if ((perms & permManageServer) === permManageServer) {
        return true
    }

    return false
}