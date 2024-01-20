import { createContext, useContext } from "react";
import { useGuilds } from "./GuildsProvider";
import { BotGuild } from "botloader-common";

export const CurrentGuildContext = createContext<BotGuild | undefined>(undefined);

export function CurrentGuildProvider(props: { guildId?: string, children: React.ReactNode }) {
    let guilds = useGuilds();

    let current = undefined;
    if (guilds && props.guildId) {
        current = guilds.all.find(g => g.guild.id === props.guildId);
    }

    return <CurrentGuildContext.Provider value={current}>{props.children}</CurrentGuildContext.Provider>
}

export function useCurrentGuild() {
    return useContext(CurrentGuildContext);
}
