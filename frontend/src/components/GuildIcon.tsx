import { guildIconUrl } from "./Util"

type Props = {
    guild: { icon?: string, name: string, id: string }
}

export function GuildIcon({ guild }: Props) {
    if (guild.icon) {
        return <img src={guildIconUrl(guild)} alt={guild.name + " icon"} className="avatar" height={64} />
    } else {
        return <div style={{ height: "64px", width: "64px", textAlign: "center", display: "flex", alignItems: "center", justifyContent: "center", backgroundColor: "#222", borderRadius: "50%" }}>
            {guild.name.split(" ").map((v) => v.charAt(0))}
        </div>
    }
}
