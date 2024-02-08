import { Alert, Button, CircularProgress, Snackbar, Typography } from "@mui/material";
import { BotGuild, isErrorResponse } from "botloader-common";
import React from "react";
import { useFetchedDataBehindGuard } from "./FetchData";
import { GuildSelectionDialog } from "./GuildSelectionDialog";
import { useGuilds } from "../modules/guilds/GuildsProvider";
import { pluginContext } from "./PluginProvider";
import { useSession } from "../modules/session/useSession";

export function AddPluginToServerButton() {
    const [open, setOpen] = React.useState(false);
    const [addingToServer, setAddingToServer] = React.useState<BotGuild | null>(null);
    const [addedToServer, setAddedToServer] = React.useState<BotGuild | null>(null);
    const [addError, setAddError] = React.useState<string | null>(null);
    const session = useSession();
    let { value: plugin } = useFetchedDataBehindGuard(pluginContext);
    const guilds = useGuilds();

    const handleClickOpen = () => {
        setOpen(true);
    };

    const handleClose = (value: BotGuild | null) => {
        if (value) {
            setAddingToServer(value);
            addToServer(value);
        }
        setOpen(false);
    };

    async function addToServer(guild: BotGuild) {
        const resp = await session.apiClient.addPluginToGuild(plugin!.id, guild.guild.id, { auto_update: true });
        setAddingToServer(null);
        if (isErrorResponse(resp)) {
            setAddError(resp.response?.description ?? "unknown error");
        } else {
            setAddedToServer(guild);
        }
    }


    return <>
        <Button
            variant="outlined"
            disabled={Boolean(addingToServer) || !Boolean(guilds)}
            onClick={handleClickOpen}>
            Add to server
        </Button>

        {addingToServer ?
            <>
                <Typography>Adding to {addingToServer.guild.name}...</Typography>
                <CircularProgress />
            </>
            : null
        }
        <GuildSelectionDialog
            open={open}
            onClose={handleClose}
            guilds={guilds.value?.hasAdmin.filter((v) => v.connected) || []}
        />
        <Snackbar open={Boolean(addedToServer)} autoHideDuration={6000} onClose={() => setAddedToServer(null)}>
            <Alert onClose={() => setAddedToServer(null)} severity="success" sx={{ width: '100%' }}>
                Added to {addedToServer?.guild.name}!
            </Alert>
        </Snackbar>
        <Snackbar open={Boolean(addError)} autoHideDuration={10000} onClose={() => setAddError(null)}>
            <Alert onClose={() => setAddError(null)} severity="error" sx={{ width: '100%' }}>
                Failed adding: {addError}
            </Alert>
        </Snackbar>
    </>

}