import { Alert, Avatar, Box, Chip, Divider, Stack, Typography } from "@mui/material";
import { Container } from "@mui/system";
import { ScriptPlugin } from "botloader-common";
import ReactMarkdown from "react-markdown";
import { AddPluginToServerButton } from "../components/AddPluginToServer";
import { BlLink } from "../components/BLLink";
import { useFetchedDataBehindGuard } from "../components/FetchData";
import { pluginContext } from "../components/PluginProvider";
import { ScriptingIde } from "../components/ScriptIde";
import { userAvatarUrl } from "../components/Util";

export function ViewPlugin() {
    let { value: plugin } = useFetchedDataBehindGuard(pluginContext);

    return <Container>
        <Typography>Plugin</Typography>
        <Typography variant="h4">{plugin.name}</Typography>
        <Stack direction={"row"} alignItems="center" spacing={1}>
            <Avatar alt={plugin.author?.username} src={userAvatarUrl(plugin.author!, 64)} />
            {plugin.author?.username}#{plugin.author?.discriminator.padStart(4, "0")}
            {plugin.author?.is_bl_staff ? <Chip label="Staff" variant="outlined" /> : null}
        </Stack>
        <Divider />
        <Typography mb={2} mt={2}>{plugin.short_description}</Typography>
        {plugin.author?.is_bl_staff ? null : <Alert severity="warning">This plugin is from a community member, you should only add plugins from people you trust.</Alert>}
        <AddPluginToServerButton />
        <ReactMarkdown>{plugin.long_description}</ReactMarkdown>
        <Divider />
        <BlLink to={`/plugins/${plugin.id}/source`}>View source</BlLink>
    </Container>
}

export function ViewPluginSource() {
    const { value: plugin } = useFetchedDataBehindGuard(pluginContext);
    const cast = plugin as ScriptPlugin;

    return <ScriptingIde
        initialSource={cast.data.published_version || undefined}
        onSave={() => { }}
        files={[]}
        isReadyOnly={true}
        isDiffEditor={false}
    >
        <Box p={1}>
            <Typography>Viewing plugin</Typography>
            <Chip variant="outlined" label={plugin.name} />
            <BlLink to={`/plugins/${plugin.id}`}>Back</BlLink>
            <Divider sx={{ mb: 1 }} />
        </Box>
    </ScriptingIde >
}