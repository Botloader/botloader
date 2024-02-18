import { Alert, Avatar, Box, Chip, Divider, Paper, Stack, Typography } from "@mui/material";
import { Container } from "@mui/system";
import ReactMarkdown from "react-markdown";
import { AddPluginToServerButton } from "../../../components/AddPluginToServer";
import { BlLink } from "../../../components/BLLink";
import { useFetchedDataBehindGuard } from "../../../components/FetchData";
import { pluginContext } from "../../../components/PluginProvider";
import { ScriptingIde } from "../../../components/ScriptIde";
import { userAvatarUrl } from "../../../components/Util";
import { useSession } from "../../../modules/session/useSession";
import { PluginIcon } from "../../../components/PluginIcon";
import { pluginImageUrl } from "../../../misc/pluginImageUrl";

export function ViewPlugin() {
    const session = useSession()
    let { value: plugin } = useFetchedDataBehindGuard(pluginContext);
    if (!plugin) {
        throw new Error("plugin value was null")
    }

    const bannerImage = plugin.images.find(v => v.kind === "Banner")
    const showcaseImages = plugin.images.filter(v => v.kind === "Showcase")

    return <Container>
        <Paper sx={{ marginTop: 1 }}>

            {bannerImage && <img
                alt="plugin uploaded banner"
                src={pluginImageUrl(plugin.id, bannerImage.image_id)}
                style={{
                    width: "100%",
                    maxHeight: "194px",
                    objectFit: "cover",
                }}
            />}
            <Box sx={{ padding: 2 }}>

                <Stack direction={"row"} alignItems={"center"} spacing={1}>
                    <PluginIcon plugin={plugin} />
                    <Typography variant="h4">{plugin.name}</Typography>
                </Stack>

                <Stack direction={"row"} alignItems="center" spacing={1} mt={1} mb={1}>
                    <Avatar alt={plugin.author?.username} sx={{ width: 32, height: 32 }} src={userAvatarUrl(plugin.author!, 64)} />
                    <Typography>{plugin.author?.username}#{plugin.author?.discriminator.padStart(4, "0")}</Typography>
                    {plugin.author?.is_bl_staff ? <Chip label="Staff" size="small" variant="outlined" /> : null}
                </Stack>

                <Divider />

                <Typography mb={2} mt={2}>{plugin.short_description}</Typography>
                {plugin.author?.is_bl_staff ? null : <Alert severity="warning">This plugin is from a community member, you should only add plugins from people you trust.</Alert>}

                <AddPluginToServerButton />

                <ReactMarkdown>{plugin.long_description}</ReactMarkdown>

                <Divider />

                {session.user?.id === plugin.author_id ? <BlLink to={`/user/plugins/${plugin.id}/`}>Edit</BlLink> : null}
                <BlLink to={`/plugins/${plugin.id}/source`}>View source</BlLink>

                <Divider />
                {showcaseImages.length > 0
                    ? <>
                        <Typography>Showcase images</Typography>
                        {showcaseImages.map(v => (
                            <img
                                key={v.image_id}
                                alt="plugin showcase"
                                src={pluginImageUrl(plugin.id, v.image_id)}
                                style={{ maxWidth: "100%" }}
                            />
                        ))}
                    </>
                    : null}
            </Box>

        </Paper>
    </Container>
}

export function ViewPluginSource() {
    const { value: plugin } = useFetchedDataBehindGuard(pluginContext);
    if (!plugin) {
        throw new Error("plugin value was null")
    }

    return <ScriptingIde
        initialSource={plugin.data.published_version || undefined}
        onSave={() => { }}
        files={[]}
        isReadyOnly={true}
        isDiffEditor={false}
    >
        <Box p={1}>
            <Typography>Viewing plugin <Chip variant="outlined" label={plugin.name} /></Typography>

            <Divider sx={{ mb: 1, mt: 1 }} />
            <BlLink fullWidth to={`/plugins/${plugin.id}`}>Back to plugin page</BlLink>
            <Divider sx={{ mb: 1 }} />
        </Box>
    </ScriptingIde >
}