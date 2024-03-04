import { createContext, useContext } from "react";
import { useGuilds } from "./GuildsProvider";
import { BotGuild } from "botloader-common";
import { FetchDataHookNotBehindGuard } from "../../components/FetchData";

export const CurrentGuildContext = createContext<Omit<FetchDataHookNotBehindGuard<BotGuild>, "setData"> | undefined>(undefined);
export const CurrentGuildIdContext = createContext<string | null>(null);

export function CurrentGuildProvider(props: { guildId?: string, children: React.ReactNode }) {
    let guilds = useGuilds();

    let current = undefined;
    if (guilds.value && props.guildId) {
        current = guilds.value.all.find(g => g.guild.id === props.guildId);
    }

    return <CurrentGuildIdContext.Provider value={props.guildId ?? null}>
        <CurrentGuildContext.Provider value={{
            ...guilds,
            value: current,
        }}>
            {props.children}
        </CurrentGuildContext.Provider>
    </CurrentGuildIdContext.Provider>
}

export function useCurrentGuild() {
    return useContext(CurrentGuildContext);
}

export function useCurrentGuildId() {
    const guildId = useContext(CurrentGuildIdContext);
    if (!guildId) {
        throw new Error("No guild id context available")
    }

    return guildId
}