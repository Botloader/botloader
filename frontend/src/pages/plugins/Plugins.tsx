import { Alert, Avatar, Box, Chip, Container, Divider, Paper, Typography } from "@mui/material";
import { Stack } from "@mui/system";
import { Plugin } from "botloader-common";
import { BlLink } from "../../components/BLLink";
import { createFetchDataContext, FetchDataGuarded, useFetchedDataBehindGuard } from "../../components/FetchData";
import { userAvatarUrl } from "../../components/Util";
import { useSession } from "../../modules/session/useSession";

let context = createFetchDataContext<Plugin[]>();

export function ViewPlugins() {
    const session = useSession();

    return <Container>
        <FetchDataGuarded
            loader={async () => await session.apiClient.getPublishedPublicPlugins()}
            context={context}>
            <InnerPage />
        </FetchDataGuarded>
    </Container>
}

function InnerPage() {
    const { value: plugins } = useFetchedDataBehindGuard(context);

    return <>
        <Stack spacing={1} mt={1}>
            {plugins.map((v) => (
                <PluginItem plugin={v} />
            ))}
        </Stack>
    </>
}

function PluginItem({ plugin }: { plugin: Plugin }) {

    return <Paper sx={{ p: 1, borderRadius: 2 }}>
        <Box display="flex" justifyContent="space-between">
            <Box>
                <Typography variant="h4">{plugin.name}</Typography>
                <Stack direction={"row"} alignItems="center" spacing={1}>
                    <Avatar alt={plugin.author?.username} src={userAvatarUrl(plugin.author!, 64)} />
                    {plugin.author?.username}#{plugin.author?.discriminator.padStart(4, "0")}
                    {plugin.author?.is_bl_staff ? <Chip label="Staff" variant="outlined" /> : null}
                </Stack>
            </Box>
            <Box display="flex" alignItems="center">
                <BlLink variant="contained" to={`/plugins/${plugin.id}`}>Open</BlLink>
            </Box>
        </Box >
        <Divider />
        <Typography mb={2} mt={2}>{plugin.short_description}</Typography>
        {plugin.author?.is_bl_staff
            ? <Alert severity="success">Trusted plugin.</Alert>
            : <Alert severity="warning">Untrusted plugin from a community member</Alert>}
    </Paper >
}
