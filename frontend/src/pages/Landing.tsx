import { Box, Button, Card, CardContent, CardMedia, Container, Divider, Typography } from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { BlLink } from '../components/BLLink';
import showcase_editor from '../img/showcase_editor.png';
import showcase_plugin_settings from '../img/showcase_plugin_settings.png';
import { ViewPlugins } from './plugins/Plugins';
import { useEffect, useState } from 'react';
import { useSession } from '../modules/session/useSession';
import { NewsItem, isErrorResponse } from 'botloader-common';
import { NewsItemComponent } from '../components/NewsItem';

export function LandingPage() {
    return <>
        <Container maxWidth={false}>
            <Box
                display={"flex"}
                flexDirection={"column"}
                alignItems={"center"}
            >
                <Grid2
                    container
                    spacing={2}
                    padding={5}
                    alignItems={"center"}
                >
                    <Grid2 xs={10}>
                        <Typography variant='h6'>A fully programmable discord bot with a plugin gallery and no hosting required!</Typography>
                        <Typography color={"text.secondary"}>
                            No hosting required, add the bot to your server and you're good to go! just create a script through the web interface or add a plugin to your server.
                        </Typography>
                    </Grid2>
                    <Grid2 xs={2}>
                        <img src="/logo192.png" alt="zzz" style={{ borderRadius: "50%", objectFit: "contain", maxWidth: "100%" }} ></img>
                    </Grid2>
                </Grid2>

                <Grid2
                    container
                    spacing={2}
                    padding={5}
                    alignItems={"stretch"}
                >
                    <Grid2 xs={12} md={6}>
                        <Card sx={{ height: "100%" }}>
                            <CardMedia
                                component="img"
                                src={showcase_plugin_settings}
                                alt="plugin showcase"
                            />
                            <CardContent>
                                <Typography gutterBottom variant="h5" component="div">
                                    Plugins
                                </Typography>
                                <Typography variant="body2" color="text.secondary">
                                    Use already crafted plugins from both the community and the bot authors to add functionality to your discord server without any coding.<br />
                                    Plugins can be configured in the web interface without touching the code.
                                </Typography>
                                <Typography mt={2} color="text.secondary">You can see a list of available plugins <BlLink to='/plugins'>On the plugin page</BlLink></Typography>
                            </CardContent>
                        </Card>
                    </Grid2>

                    <Grid2 xs={12} md={6} >
                        <Card sx={{ height: "100%" }}>
                            <CardMedia
                                component="img"
                                src={showcase_editor}
                                alt="editor showcase"
                            />
                            <CardContent>
                                <Typography gutterBottom variant="h5" component="div">
                                    Fully Programmable
                                </Typography>
                                <Typography variant="body2" color="text.secondary">
                                    Create custom scripts for your server in TypeScript using either the online editor or the vs code extension.<br />
                                    We provide API's for storage, timers, scheduled tasks, configuration and more!
                                </Typography>
                            </CardContent>
                        </Card>
                    </Grid2>
                    <Grid2 xs={12} marginTop={5} >
                        <Box justifyContent={"center"} display={"flex"} flexDirection={"column"} alignItems={"center"}>
                            <LatestNewsItem />
                        </Box>
                    </Grid2>
                </Grid2>
            </Box>


            <div style={{ display: "flex", justifyContent: "center" }}>
                <BlLink to="/servers">Control panel</BlLink>
                <Button href="https://discord.gg/HJM3MqVBfw">Discord server</Button>
                <Button href="/docs/">Documentation</Button>
            </div>
        </Container >
        <Divider sx={{ marginTop: 1 }} />
        <Typography variant='h3' align='center'>Plugin Showcase</Typography>
        <ViewPlugins />
    </>
}

function LatestNewsItem() {
    const session = useSession();
    const [news, setNews] = useState<undefined | null | NewsItem[]>(undefined);

    useEffect(() => {
        async function fetchNews() {
            let resp = await session.apiClient.getNews();
            if (isErrorResponse(resp)) {
                setNews(null);
            } else {
                setNews(resp);
            }
        }

        fetchNews();
    }, [session])

    if (!news || news.length < 1) {
        return <></>
    }

    return <NewsItemComponent item={news[0]} />

}