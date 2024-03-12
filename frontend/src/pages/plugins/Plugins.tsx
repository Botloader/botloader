import {
    Alert,
    Avatar,
    Box,
    CardContent,
    CardHeader,
    CardMedia,
    Container,
    Paper,
    Typography
} from "@mui/material";
import { Stack } from "@mui/system";
import { Plugin } from "botloader-common";
import { BlLink } from "../../components/BLLink";
import { createFetchDataContext, FetchDataGuarded, useFetchedDataBehindGuard } from "../../components/FetchData";
import { userAvatarUrl } from "../../components/Util";
import { useSession } from "../../modules/session/useSession";
import { PluginIcon } from "../../components/PluginIcon";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import { pluginImageUrl } from "../../misc/pluginImageUrl";

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

    return <Grid2 container spacing={1} mt={1} justifyContent={"center"}>
        {plugins.map((v) => (
            <Grid2 key={v.id}>
                <PluginItem plugin={v} />
            </Grid2>
        ))}
    </Grid2>
}

function PluginItem({ plugin }: { plugin: Plugin }) {
    const bannerImage = plugin.images.find(v => v.kind === "Banner")

    return <Paper
        sx={{
            maxHeight: 400,
            height: 400,
            minWidth: 345,
            maxWidth: 345,
            display: "flex",
            flexDirection: "column",
            justifyContent: "start"
        }}
    >
        <CardHeader
            avatar={
                <PluginIcon plugin={plugin} />
            }
            title={plugin.name}
            titleTypographyProps={{
                variant: "h5",
            }}
            subheader={<>{plugin.author
                ? <Stack mt={1} direction={"row"} alignItems={"center"} gap={1}>
                    <Avatar
                        alt={plugin.author.username}
                        src={userAvatarUrl(plugin.author, 64)}
                        sx={{ width: 32, height: 32 }}
                    />
                    {plugin.author.username}
                </Stack>
                : "unknown user"}</>
            }
        />
        {bannerImage && <CardMedia
            component="img"
            image={pluginImageUrl(plugin.id, bannerImage.image_id)}
            alt={plugin.name + " banner"}
            sx={{
                minHeight: 10,
                flexShrink: 1,
            }}
        />}

        <CardContent sx={{
            flexGrow: 1
        }}>
            <Typography mb={1} color="text.secondary">
                {plugin.short_description}
            </Typography>
        </CardContent>

        <Stack
            direction={"row"}
            alignItems={"center"}
            justifyContent={"space-between"}
            padding={1}
            justifySelf={"flex-end"}
        >
            {/* <CardContent> */}
            <Box flexGrow={1} marginRight={1}>
                {plugin.author?.is_bl_staff
                    ? <Alert severity="success">Official plugin</Alert>
                    : <Alert severity="info">Community Plugin</Alert>}
            </Box>

            {/* </CardContent> */}
            {/* <CardActions sx={{ justifyContent: "center" }} > */}
            <BlLink variant="contained" to={`/plugins/${plugin.id}`}>Open</BlLink>
            {/* </CardActions> */}
        </Stack>
    </Paper>
}
