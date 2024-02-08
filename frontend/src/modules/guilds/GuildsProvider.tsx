import { ApiResult, BotGuild, isErrorResponse, UserGuild } from "botloader-common";
import { createFetchDataContext, FetchData, FetchDataGuard, useFetchedData } from "../../components/FetchData";
import { useSession } from "../session/useSession";

export const GuildsContext = createFetchDataContext<LoadedGuilds>();

export function GuildsProvider({ children }: { children: React.ReactNode }) {
    const session = useSession();

    async function fetch(): Promise<ApiResult<LoadedGuilds> | undefined> {
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
    }

    return <FetchData loader={fetch} context={GuildsContext}>
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