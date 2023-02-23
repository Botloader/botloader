import { Alert, Box, CircularProgress, Divider, Paper, Snackbar, Stack, TextField } from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import { BotGuild, isErrorResponse, ScriptPlugin } from "botloader-common";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { DisplayDateTime } from "../../components/DateTime";
import { useFetchedDataBehindGuard } from "../../components/FetchData";
import { GuildsGuard, useGuilds } from "../../components/GuildsProvider";
import { pluginContext } from "../../components/PluginProvider";
import { useSession } from "../../components/Session";
import * as React from 'react';
import Button from '@mui/material/Button';
import ListItem from '@mui/material/ListItem';
import Typography from '@mui/material/Typography';
import { GuildSelectionDialog } from "../../components/GuildSelectionDialog";


export function EditPluginPage() {
    let { value: plugin } = useFetchedDataBehindGuard(pluginContext);

    return <Box p={1}>
        <Typography>Editing plugin</Typography>
        <Typography variant="h3">{plugin.name}</Typography>
        <Divider sx={{ mb: 2 }} />
        <Typography variant="h4">General settings</Typography>
        <Paper sx={{ p: 1 }}>
            <EditPluginMetaForm />
        </Paper>

        <Grid2 container spacing={2} mt={1}>
            {plugin.data.plugin_type === "ScriptPlugin" ? <Grid2 xs={6}><ScriptPluginSettings /></Grid2> : null}
            <Grid2 xs>
                <GuildsGuard>
                    <AddToServer />
                </GuildsGuard>
            </Grid2>
        </Grid2>
    </Box>
}


function EditPluginMetaForm() {
    const session = useSession();
    let { value: plugin } = useFetchedDataBehindGuard(pluginContext);

    const [name, setName] = useState(plugin.name);
    const [shortDescription, setShortDescription] = useState(plugin.short_description);
    const [longDescription, setLongDescription] = useState(plugin.long_description);

    const [isSaving, setSaving] = useState(false);

    const [errors, setErrors] = useState<{ name?: string, short?: string, long?: string, general?: string }>({})
    const [saveNotifOpen, setSaveNotifOpen] = useState(false);

    async function save() {
        setSaving(true);
        setErrors({});

        let result = await session.apiClient.updatePluginMeta(plugin.id, {
            name: name,
            short_description: shortDescription,
            long_description: longDescription,
        })

        if (isErrorResponse(result)) {
            setErrors({
                general: result.response?.description,
                name: result.getFieldError("name"),
                short: result.getFieldError("short_description"),
                long: result.getFieldError("long_description"),
            })
            setSaving(false);
        } else {
            setSaving(false);
            setSaveNotifOpen(true);
        }
    }

    return <Stack direction={"column"} spacing={2}>
        <TextField label="Name" variant="outlined"
            error={Boolean(errors.name)} helperText={errors.name}
            onChange={(evt) => setName(evt.target.value)} value={name} />
        <TextField label="Short description" variant="outlined"
            error={Boolean(errors.short)} helperText={errors.short}
            onChange={(evt) => setShortDescription(evt.target.value)} value={shortDescription} />
        <TextField label="Long description" multiline variant="outlined"
            error={Boolean(errors.long)} helperText={errors.long}
            onChange={(evt) => setLongDescription(evt.target.value)} value={longDescription} />

        <Typography variant="body1" color={"error"}>{errors.general}</Typography>
        <Button disabled={isSaving} color="success" onClick={() => save()}>Save!</Button>
        <Snackbar open={saveNotifOpen} autoHideDuration={6000} onClose={() => setSaveNotifOpen(false)}>
            <Alert onClose={() => setSaveNotifOpen(false)} severity="success" sx={{ width: '100%' }}>
                Saved Settings!
            </Alert>
        </Snackbar>
    </Stack>
}

function ScriptPluginSettings() {
    const { value: plugin, reload } = useFetchedDataBehindGuard(pluginContext);
    const session = useSession();
    const cast = plugin as ScriptPlugin;
    const navigate = useNavigate();

    async function publish() {
        await session.apiClient.publishScriptPluginVersion(plugin.id, { source: cast.data.dev_version ?? "" });
        reload();
    }

    return <Paper>
        <ListItem>
            {cast.data.published_version
                ? <Typography>Last version published at: <DisplayDateTime dt={cast.data.published_version_updated_at!} /> </Typography>
                : <Typography>You have never published a version of this plugin.</Typography>}
        </ListItem>

        <ListItem>
            {cast.data.dev_version
                ? <Typography>Last development version updated at: <DisplayDateTime dt={cast.data.dev_version_updated_at!} /> </Typography>
                : <Typography>You have done zero development yet on this plugin :(</Typography>}

            <Button onClick={() => navigate(`/user/plugins/${plugin.id}/edit_script_diff`)}>View changes</Button>
            <Button onClick={publish}>Publish</Button>
        </ListItem>

        <ListItem>
            <Button onClick={() => navigate(`/user/plugins/${plugin.id}/edit_script`)}>Edit development version</Button>
        </ListItem>

    </Paper>
}

function AddToServer() {
    const [open, setOpen] = React.useState(false);
    const [addingToServer, setAddingToServer] = React.useState<BotGuild | null>(null);
    const [addedToServer, setAddedToServer] = React.useState<BotGuild | null>(null);
    const [addError, setAddError] = React.useState<string | null>(null);
    const session = useSession();
    let { value: plugin } = useFetchedDataBehindGuard(pluginContext);


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
        const resp = await session.apiClient.addPluginToGuild(plugin.id, guild.guild.id, { auto_update: true });
        setAddingToServer(null);
        if (isErrorResponse(resp)) {
            setAddError(resp.response?.description ?? "unknown error");
        } else {
            setAddedToServer(guild);
        }
    }

    const guilds = useGuilds()!;

    return (
        <Paper sx={{ p: 1 }}>
            <Typography>Add this plugin to a server</Typography>
            <Button disabled={Boolean(addingToServer)} onClick={handleClickOpen}>
                Select server
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
                guilds={guilds.hasAdmin.filter((v) => v.connected)}
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
        </Paper>
    );
}