import { guildIconUrl } from "./Util"

type Props = {
    guild: { icon?: string, name: string, id: string }
    discordSize?: 32 | 64 | 128,
    size?: number,
}

export function GuildIcon({ guild, size, discordSize }: Props) {
    let resolvedDiscordSize = discordSize ?? 64;
    let resolvedSize = size ?? 64;
    if (guild.icon) {
        return <img src={guildIconUrl(guild, resolvedDiscordSize)} alt={guild.name + " icon"} className="avatar" height={resolvedSize} />
    } else {
        return <div style={{ height: resolvedSize + "px", width: resolvedSize + "px", textAlign: "center", display: "flex", alignItems: "center", justifyContent: "center", backgroundColor: "#222", borderRadius: "50%" }}>
            {guild.name.split(" ").map((v) => v.charAt(0))}
        </div>
    }
}