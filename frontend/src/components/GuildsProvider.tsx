import { createContext, useContext, useEffect, useState } from "react";
import { BotGuild, CurrentGuildsResponse, isErrorResponse } from "botloader-common";
import { useSession } from "./Session";

export const GuildsContext = createContext<CurrentGuildsResponse | undefined>(undefined);

export function GuildsProvider({ children }: { children: React.ReactNode }) {
    let session = useSession();
    let [guilds, setGuilds] = useState<CurrentGuildsResponse | undefined>(undefined);

    useEffect(() => {
        async function inner() {
            let guilds = await session.apiClient.getCurrentUserGuilds();
            if (!isErrorResponse(guilds)) {
                setGuilds(guilds);
            }
        }

        if (session.user) {
            console.log("YOOOOOOOOOOOOOOOOOOOOOOOOO");
            inner();
        } else {
            console.log("no user, not fetching guilds");
        }
    }, [session])


    return <GuildsContext.Provider value={guilds}>{children}</GuildsContext.Provider>
}

export function useGuilds() {
    return useContext(GuildsContext)
}

export const CurrentGuildContext = createContext<BotGuild | undefined>(undefined);

export function CurrentGuildProvider(props: { guildId?: string, children: React.ReactNode }) {
    let guilds = useGuilds();

    let current = undefined;
    if (guilds && props.guildId) {
        current = guilds.guilds.find(g => g.guild.id === props.guildId);
    }

    return <CurrentGuildContext.Provider value={current}>{props.children}</CurrentGuildContext.Provider>
}

export function useCurrentGuild() {
    return useContext(CurrentGuildContext);
}