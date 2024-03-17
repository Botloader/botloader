import { Box, Button, Container, Divider, Typography } from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { BlLink } from '../components/BLLink';
import showcase_editor from '../img/showcase_editor.png';
import showcase_plugins from '../img/showcase_plugins.png';
import showcase_plugin_settings from '../img/showcase_plugin_settings.png';
import { ViewPlugins } from './plugins/Plugins';

export function LandingPage() {
    return <>
        <Container maxWidth={false}>
            <Box
                display={"flex"}
                flexDirection={"column"}
                alignItems={"center"}
            >
                <Box sx={{
                    backgroundColor: "rgba(84, 127, 255, 0.1)",
                    padding: 2,
                    borderRadius: 3,
                    marginTop: 1,
                }}>
                    <Typography variant='h6'>A fully programmable discord bot with a plugin gallery and no hosting required</Typography>
                    <p style={{ backgroundColor: "#378855", borderRadius: "10px", padding: "10px", textAlign: "center" }}>Verified and in beta!</p>
                </Box>

                <Grid2
                    container
                    padding={2}
                    alignItems={"center"}
                    marginTop={1}
                >
                    <Grid2 sm={6} xs={12}>
                        <Typography variant='h6'>Plugins</Typography>
                        <Typography color={"text.secondary"}>
                            Use already crafted plugins from both the community and the bot authors to add functionality to your discord server without any coding.<br />
                            Plugins can be configured in the web interface without touching the code.
                        </Typography>
                        <Typography>You can see a list of available plugins <BlLink to='/plugins'>On the plugin page</BlLink></Typography>
                    </Grid2>
                    <Grid2 sm={6} xs={12} overflow={"hidden"}>
                        <img src={showcase_plugin_settings} alt="editor_screenshot" style={{
                            borderRadius: "10px",
                            marginTop: "20px",
                            objectFit: "contain",
                            maxWidth: "100%",
                        }}></img>
                    </Grid2>
                    {/* <Grid2 sm={3} xs={12} overflow={"hidden"}>
                        <img src={showcase_plugins} alt="editor_screenshot" style={{
                            borderRadius: "10px",
                            marginTop: "20px",
                            objectFit: "contain",
                            maxWidth: "100%",
                        }}></img>
                    </Grid2> */}
                </Grid2>

                <Grid2
                    container
                    spacing={2}
                    padding={5}
                    alignItems={"center"}
                >
                    <Grid2 sm={6} xs={12}>
                        <Typography variant='h6'>Fully Programmable</Typography>
                        <Typography color={"text.secondary"}>
                            Create custom scripts for your server in TypeScript using either the online editor or the vs code extension.<br />
                            We provide API's for storage, timers, scheduled tasks, configuration and more!
                        </Typography>
                    </Grid2>
                    <Grid2 sm={6} xs={12} overflow={"hidden"}>
                        <img src={showcase_editor} alt="editor_screenshot" style={{
                            borderRadius: "10px",
                            marginTop: "20px",
                            objectFit: "contain",
                            maxWidth: "100%",
                        }}></img>
                    </Grid2>
                </Grid2>

                <Grid2
                    container
                    spacing={2}
                    padding={5}
                    alignItems={"center"}
                >
                    <Grid2 xs={10}>
                        <Typography variant='h6'>No hosting, setup or installing required!</Typography>
                        <Typography color={"text.secondary"}>
                            All you have to do is add the bot to your server and you're good to go! just create a script through the web interface or add a plugin to your server.
                        </Typography>
                    </Grid2>
                    <Grid2 xs={2}>
                        <img src="/logo192.png" alt="zzz" style={{ borderRadius: "50%", objectFit: "contain", maxWidth: "100%" }} ></img>
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