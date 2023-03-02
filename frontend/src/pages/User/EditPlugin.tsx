import { Box, Checkbox, Divider, FormControlLabel, Paper, Stack, TextField } from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import { isErrorResponse, ScriptPlugin } from "botloader-common";
import { useState } from "react";
import { DisplayDateTime } from "../../components/DateTime";
import { useFetchedDataBehindGuard } from "../../components/FetchData";
import { GuildsGuard } from "../../components/GuildsProvider";
import { pluginContext } from "../../components/PluginProvider";
import { useSession } from "../../components/Session";
import * as React from 'react';
import Button from '@mui/material/Button';
import ListItem from '@mui/material/ListItem';
import Typography from '@mui/material/Typography';
import { BlLink } from "../../components/BLLink";
import { UseNotifications } from "../../components/Notifications";
import { AddPluginToServerButton } from "../../components/AddPluginToServer";


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
    const [isPublic, setIsPublic] = useState(plugin.is_public);

    const [isSaving, setSaving] = useState(false);

    const [errors, setErrors] = useState<{ name?: string, short?: string, long?: string, general?: string }>({})
    const notifications = UseNotifications();

    async function save() {
        setSaving(true);
        setErrors({});

        let result = await session.apiClient.updatePluginMeta(plugin.id, {
            name: name,
            short_description: shortDescription,
            long_description: longDescription,
            is_public: isPublic,
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

            notifications.push({ class: "success", message: "Updated plugin settings!" })
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

        <FormControlLabel control={<Checkbox checked={isPublic} onChange={(_, v) => setIsPublic(v)} />} label="Public" />
        <Typography variant="body1">Public plugins can be added by anyone and may appear in the public plugin directory</Typography>

        <Typography variant="body1" color={"error"}>{errors.general}</Typography>
        <Button disabled={isSaving} color="success" onClick={() => save()}>Save!</Button>
    </Stack>
}

function ScriptPluginSettings() {
    const { value: plugin, reload } = useFetchedDataBehindGuard(pluginContext);
    const session = useSession();
    const cast = plugin as ScriptPlugin;

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
        </ListItem>

        <ListItem>
            {cast.data.published_version !== cast.data.dev_version
                ? <>
                    <Typography>This plugin has unpublished changes!</Typography>
                    <BlLink to={`/user/plugins/${plugin.id}/edit_script_diff`}>View changes</BlLink>
                    <Button onClick={publish}>Publish</Button>
                </>
                : <Typography>No unpublished changes</Typography>}
        </ListItem>

        <ListItem>
            <BlLink to={`/user/plugins/${plugin.id}/edit_script`}>Open editor</BlLink>
        </ListItem>

    </Paper >
}

function AddToServer() {
    let { value: plugin } = useFetchedDataBehindGuard(pluginContext);
    return (
        <Paper sx={{ p: 1 }}>
            <AddPluginToServerButton />
            <BlLink to={`/plugins/${plugin.id}`} disabled={Boolean(plugin.is_public)}>
                View public page
            </BlLink>
        </Paper>
    );
}