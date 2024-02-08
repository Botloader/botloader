import { createContext, useContext } from "react";
import { useGuilds } from "./GuildsProvider";
import { BotGuild } from "botloader-common";
import { FetchDataHookNotBehindGuard } from "../../components/FetchData";

export const CurrentGuildContext = createContext<Omit<FetchDataHookNotBehindGuard<BotGuild>, "setData"> | undefined>(undefined);

export function CurrentGuildProvider(props: { guildId?: string, children: React.ReactNode }) {
    let guilds = useGuilds();

    let current = undefined;
    if (guilds.value && props.guildId) {
        current = guilds.value.all.find(g => g.guild.id === props.guildId);
    }

    return <CurrentGuildContext.Provider value={{
        ...guilds,
        value: current,
    }}>{props.children}</CurrentGuildContext.Provider>
}

export function useCurrentGuild() {
    return useContext(CurrentGuildContext);
}
