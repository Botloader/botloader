import { guildIconUrl } from "./Util"

type Props = {
    guild: { icon?: string, name: string, id: string }
    size?: 32 | 64,
}

export function GuildIcon({ guild, size }: Props) {
    let resolvedSize = size ?? 64;
    if (guild.icon) {
        return <img src={guildIconUrl(guild)} alt={guild.name + " icon"} className="avatar" height={resolvedSize} />
    } else {
        return <div style={{ height: resolvedSize + "px", width: resolvedSize + "px", textAlign: "center", display: "flex", alignItems: "center", justifyContent: "center", backgroundColor: "#222", borderRadius: "50%" }}>
            {guild.name.split(" ").map((v) => v.charAt(0))}
        </div>
    }
}
