import {
    Alert,
    Avatar,
    Box,
    Checkbox,
    Dialog,
    DialogActions,
    DialogContent,
    DialogContentText,
    DialogTitle,
    Divider,
    FormControlLabel,
    Paper,
    Stack,
    TextField
} from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import { isErrorResponse, PluginImageKind, ScriptPlugin } from "botloader-common";
import { useState } from "react";
import { DisplayDateTime } from "../../../../components/DateTime";
import { useFetchedDataBehindGuard } from "../../../../components/FetchData";
import { GuildsGuard } from "../../../../modules/guilds/GuildsProvider";
import { pluginContext } from "../../../../components/PluginProvider";
import * as React from 'react';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import { BlLink } from "../../../../components/BLLink";
import { UseNotifications } from "../../../../components/Notifications";
import { AddPluginToServerButton } from "../../../../components/AddPluginToServer";
import { useSession } from "../../../../modules/session/useSession";
import { PluginIcon } from "../../../../components/PluginIcon";
import { AsyncOpButton } from "../../../../components/AsyncOpButton";
import { pluginImageUrl } from "../../../../misc/pluginImageUrl";

export function EditPluginPage() {
    const [iconOptionOpen, setIconOptionsOpen] = useState(false)
    const { value: plugin } = useFetchedDataBehindGuard(pluginContext);

    return <Box p={1}>
        <Typography>Editing plugin</Typography>
        <Stack direction={"row"} alignItems={"center"} padding={1}>
            <Button onClick={() => setIconOptionsOpen(true)}>
                <PluginIcon plugin={plugin} />
            </Button>
            <Divider orientation="vertical" sx={{ ml: 1, mr: 1, alignSelf: "stretch", height: "unset" }}></Divider>
            <Typography variant="h3">{plugin.name}</Typography>
        </Stack>
        <GuildsGuard>
            <AddPluginToServerButton />
            <BlLink to={`/plugins/${plugin.id}`} disabled={!Boolean(plugin.is_public)}>
                View public page
            </BlLink>
        </GuildsGuard>
        <Divider sx={{ mb: 2, mt: 2 }} />

        <Grid2 container spacing={1}>
            <Grid2 xs={6}>
                <Typography variant="h4">General settings</Typography>
                <Paper sx={{ p: 2 }}>
                    <EditPluginMetaForm />
                </Paper>
            </Grid2>

            <Grid2 xs={6}>
                <Stack>
                    {plugin.data.plugin_type === "ScriptPlugin" ? <ScriptPluginSettings /> : null}
                </Stack>
            </Grid2>
        </Grid2>

        <Grid2 container spacing={1}>
            <Grid2 xs={12} md={6}>
                <ShowcaseImages />
            </Grid2>
            <Grid2 xs={12} md={6}>
                <BannerImage />
            </Grid2>
        </Grid2>

        <IconDialog isOpen={iconOptionOpen} close={() => setIconOptionsOpen(false)} />
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

    return <>
        <Typography variant="h4">Development</Typography>
        <Paper sx={{ p: 2 }}>
            {cast.data.published_version
                ? <Typography>Last version published at: <DisplayDateTime dt={cast.data.published_version_updated_at!} /> </Typography>
                : <Typography>You have never published a version of this plugin.</Typography>}

            {cast.data.dev_version
                ? <Typography>Last development version updated at: <DisplayDateTime dt={cast.data.dev_version_updated_at!} /> </Typography>
                : <Typography>You have done zero development yet on this plugin :(</Typography>}

            {cast.data.published_version !== cast.data.dev_version
                ? <Alert severity="info">
                    <Typography>This plugin has unpublished changes!</Typography>
                    <BlLink to={`/user/plugins/${plugin.id}/edit_script_diff`}>View changes</BlLink>
                    <Button onClick={publish}>Publish</Button>
                </Alert>
                : <Alert severity="success">
                    Plugin is up to date
                </Alert>}

            <BlLink to={`/user/plugins/${plugin.id}/edit_script`}>Open editor</BlLink>
        </Paper >
    </>
}

function IconDialog({ isOpen, close }: { isOpen: boolean, close: () => any }) {
    let { value: plugin, reload } = useFetchedDataBehindGuard(pluginContext);
    let notifications = UseNotifications();
    const session = useSession()

    const currentImage = plugin.images.find(v => v.kind === "Icon")

    async function deleteExisting() {
        if (currentImage) {
            let resp = await session.apiClient.deletePluginImage(plugin.id, currentImage.image_id)
            if (isErrorResponse(resp)) {
                notifications.push({
                    class: "error",
                    message: "Failed deleting image"
                })
            } else {
                notifications.push({
                    class: "success",
                    message: "Successfully deleted icon"
                })

                reload()
                close()
            }
        }
    }

    return <Dialog open={isOpen}>
        <DialogTitle>Plugin icon settings</DialogTitle>
        <DialogContent>
            <DialogContentText >
                <AsyncOpButton onClick={deleteExisting} disabled={!Boolean(currentImage)} label="Delete existing"></AsyncOpButton>
                {/* <Button color="info" onClick={deleteExisting}>Delete existing</Button> */}
            </DialogContentText >
            <Divider sx={{ marginBottom: 1 }} />
            <DialogContentText mb={2}>
                Change your plugin icon
            </DialogContentText>

            <UploadImage kind="Icon" onSaved={() => close()} />
        </DialogContent>
        <DialogActions>
            <Button onClick={close}>Cancel</Button>
            {/* <Button type="submit">Subscribe</Button> */}
        </DialogActions>
    </Dialog>
}

function ShowcaseImages() {
    const { value: plugin, reload } = useFetchedDataBehindGuard(pluginContext);
    const session = useSession()
    const notifications = UseNotifications()

    async function deleteImage(id: string) {
        let resp = await session.apiClient.deletePluginImage(plugin.id, id)
        if (isErrorResponse(resp)) {
            notifications.push({
                class: "error",
                message: "Failed deleting image"
            })
        } else {
            notifications.push({
                class: "success",
                message: "Successfully deleted showcase image"
            })

            reload()
        }
    }

    return <>
        <Typography variant="h4">Showcase images</Typography>
        <Paper sx={{ padding: 2 }}>
            <Paper elevation={3} sx={{ marginBottom: 1, maxWidth: 300, padding: 1 }}>
                <Typography>Upload new</Typography>
                <UploadImage kind="Showcase" />
            </Paper>


            {plugin.images.filter(v => v.kind === "Showcase").map(image => (
                <Paper elevation={2} sx={{ padding: 1 }} key={image.image_id}>
                    <Stack alignItems={"center"} width={300}>
                        <img width={300} alt={`uploaded asset`} src={pluginImageUrl(plugin.id, image.image_id)}></img>
                        <AsyncOpButton
                            onClick={() => deleteImage(image.image_id)}
                            label="Delete">
                        </AsyncOpButton>
                    </Stack>
                </Paper>
            ))}
        </Paper>

    </>
}

function UploadImage({
    onSaved,
    kind,
}: {
    onSaved?: () => any,
    kind: PluginImageKind,
}) {
    const { value: plugin, reload } = useFetchedDataBehindGuard(pluginContext);
    const [newSelectedImage, setSelectedImage] = useState<{
        file: File,
        url: string,
    } | null>(null)
    const notifications = UseNotifications()
    const session = useSession()

    React.useEffect(() => {
        return () => {
            console.log("Running unmount thing", newSelectedImage?.file.name)
            if (newSelectedImage) {
                URL.revokeObjectURL(newSelectedImage.url)
            }
        }
    }, [newSelectedImage])

    async function save() {
        if (!newSelectedImage) {
            return
        }

        const formData = new FormData()

        formData.set("form", JSON.stringify({
            kind,
        }))
        formData.append("image", newSelectedImage.file)

        let resp = await session.apiClient.addPluginImage(plugin.id, formData)
        if (isErrorResponse(resp)) {
            notifications.push({
                class: "error",
                message: resp.response?.description ?? "failed saving"
            })
        } else {
            notifications.push({
                class: "success",
                message: "Successfully uploaded image"
            })

            onSaved && onSaved()
            reload()
            setSelectedImage(null)
        }
    }

    return <Stack direction={"column"} spacing={2}>
        <input type="file" accept="image/*" onChange={(evt) => {
            if (evt.target.files) {
                const file = evt.target.files.item(0)
                if (file) {
                    const url = URL.createObjectURL(file)
                    setSelectedImage((old) => {
                        if (old) {
                            URL.revokeObjectURL(old.url)
                        }

                        return {
                            file,
                            url,
                        }
                    })
                }
            }
        }} />
        {newSelectedImage ? <>
            <Typography>Previews</Typography>
            <Avatar src={newSelectedImage?.url} variant="rounded">?</Avatar>
            <img src={newSelectedImage?.url} alt="upload preview" width={256} />
            <AsyncOpButton label="Upload" onClick={save} />
        </> : null}
    </Stack>

}

function BannerImage() {
    const { value: plugin, reload } = useFetchedDataBehindGuard(pluginContext);
    const session = useSession()
    const notifications = UseNotifications()

    const currentImage = plugin.images.find(v => v.kind === "Banner")

    async function deleteImage() {
        if (!currentImage) {
            return
        }

        let resp = await session.apiClient.deletePluginImage(plugin.id, currentImage.image_id)
        if (isErrorResponse(resp)) {
            notifications.push({
                class: "error",
                message: "Failed deleting image"
            })
        } else {
            notifications.push({
                class: "success",
                message: "Successfully deleted banner image"
            })

            reload()
        }
    }

    return <>
        <Typography variant="h4">Banner Image</Typography>
        <Paper sx={{ padding: 2 }}>
            <Paper elevation={3} sx={{ marginBottom: 1, maxWidth: 300, padding: 1 }}>
                <Typography>Upload new</Typography>
                <UploadImage kind="Banner" />
            </Paper>

            {currentImage ?
                <Paper elevation={2} sx={{ padding: 1 }} key={currentImage.image_id}>
                    <Stack alignItems={"center"} width={300}>
                        <img width={300} alt={`uploaded asset`} src={pluginImageUrl(plugin.id, currentImage.image_id)}></img>
                        <AsyncOpButton
                            onClick={deleteImage}
                            label="Delete">
                        </AsyncOpButton>
                    </Stack>
                </Paper>
                : null}
        </Paper>

    </>
}