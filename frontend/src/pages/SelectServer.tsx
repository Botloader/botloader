import { useMemo } from "react";
import { Link } from "react-router-dom";
import { BotGuild, UserGuild } from "botloader-common";
import { useGuilds } from "../components/GuildsProvider"
import { guildIconUrl } from "../components/Util";
import "./SelectServer.css"
import { Panel } from "../components/Panel";
import { Alert, Button, Card, CardActions, CardContent, CardHeader, CardMedia, Container, Skeleton, Typography } from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import { height } from "@mui/system";

export function SelectServerPage() {

    const guilds = useGuilds();

    const [joinedHasAdmin, notJoinedHasAdmin] = useMemo(() => {
        if (!guilds) {
            return [[], []];
        }

        const guildsAdmins = guilds.guilds.filter(g => hasAdmin(g.guild));
        const joinedHasAdmin = guildsAdmins.filter(g => g.connected);
        const notJoinedHasAdmin = guildsAdmins.filter(g => !g.connected);

        return [joinedHasAdmin, notJoinedHasAdmin];

    }, [guilds])

    if (!guilds) {
        return <p>Loading guilds.... (unless you're not logged in that is)</p>
    }

    return <Container>
        <Alert severity="warning">Botloader is still in early development, beware of bugs and potential breaking changes.</Alert>

        <h2>Joined servers</h2>
        <Grid2 container gap={2}>
            {joinedHasAdmin.map(g => <GuildListItem guild={g} key={g.guild.id} />)}
        </Grid2>

        <h2>Add to new servers</h2>
        <Grid2 container gap={2}>
            {notJoinedHasAdmin.map(g => <GuildListItem guild={g} key={g.guild.id} />)}
        </Grid2>
    </Container>
}

function GuildListItem({ guild: g }: { guild: BotGuild }) {
    return <Grid2>
        <Link to={`/servers/${g.guild.id}`}><Button variant="outlined" startIcon={<GuildIcon guild={g.guild}></GuildIcon>}>{g.guild.name}</Button></Link>
    </Grid2>
}

function GuildIcon(props: { guild: UserGuild }) {
    if (props.guild.icon) {
        return <img src={guildIconUrl(props.guild)} alt={props.guild.name + " icon"} className="avatar" height={64} />
    } else {
        return <div style={{ height: "64px", width: "64px", textAlign: "center", display: "flex", alignItems: "center", justifyContent: "center", backgroundColor: "#222", borderRadius: "50%" }}>
            {props.guild.name.split(" ").map((v) => v.charAt(0))}
        </div>
    }
}

const permAdmin = BigInt("0x0000000008");
const permManageServer = BigInt("0x0000000020");

function hasAdmin(g: UserGuild): boolean {
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

function shorten(name: string): string {
    const maxLength = 35
    return name.length > maxLength ? name.slice(0, 34) + '...' : name
}
