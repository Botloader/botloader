import {
    Alert,
    Avatar,
    Box,
    CardContent,
    CardHeader,
    CardMedia,
    Chip,
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
import VerifiedIcon from '@mui/icons-material/Verified';
import PeopleIcon from '@mui/icons-material/People';
import TimelineIcon from '@mui/icons-material/Timeline';
import { DisplayRelativeDateTime } from "../../components/DateTime";
import { AddPluginToServerButton } from "../../components/AddPluginToServer";

let context = createFetchDataContext<Plugin[]>();

export function ViewPlugins() {
    const session = useSession();

    return <Container maxWidth={false}>
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
        <Box display={"flex"}>
            <CardHeader
                sx={{
                    flexGrow: 1,
                    paddingTop: 1,
                    paddingBottom: 1
                }}
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


        </Box>

        <Box padding={1} display={"flex"} flexDirection={"row"} gap={1} alignItems={"center"} >
            {plugin.author?.is_bl_staff
                ? <Chip label="Official" size="small" color="success" variant="outlined" icon={<VerifiedIcon />} />
                : <Chip label="Community Plugin" size="small" color="default" variant="outlined" icon={<PeopleIcon />} />}

            <Chip label={`${plugin.installed_guilds ?? "?"} Server` + (plugin.installed_guilds !== 1 ? "s" : "")} size="small" color="primary" variant="outlined" icon={<TimelineIcon />} />

            <Typography fontSize={"0.8em"}>{plugin.published_version_updated_at &&
                <>updated <DisplayRelativeDateTime dt={plugin.published_version_updated_at} /></>
            }</Typography>
        </Box>

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
            justifyContent={"space-around"}
            padding={1}
            gap={1}
            justifySelf={"flex-end"}
        >
            <BlLink variant="outlined" fullWidth to={`/plugins/${plugin.id}`}>View</BlLink>
            <AddPluginToServerButton buttonProps={{ fullWidth: true }} plugin={plugin} />
        </Stack>
    </Paper>
}