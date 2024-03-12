import { Box, Button, Container, Divider, Paper, Typography } from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { BlLink } from '../components/BLLink';
import pogshowcase from '../img/pogshowcase.png';
import { ViewPlugins } from './plugins/Plugins';

export function LandingPage() {
    return <>
        <Container>
            <p style={{ backgroundColor: "#378855", borderRadius: "10px", padding: "10px", textAlign: "center" }}>Verified and in beta!</p>
            <Paper>
                <Grid2 container spacing={2} padding={2} alignItems={"center"}>
                    <Grid2 xs={10} display={"flex"} flexDirection={"column"} gap={2}>
                        <Box>
                            <Typography variant='h6'>A fully programmable discord bot with a plugin gallery and no hosting required.</Typography>
                        </Box>
                        <Box>
                            <Typography>Plugins</Typography>
                            <Typography color={"text.secondary"}>
                                Use already crafted plugins from both the community and the bot authors to add functionality to your discord server without any coding.<br />
                                Plugins can be configured in the web interface without touching the code.
                            </Typography>
                        </Box>
                        <Box>
                            <Typography >Fully Programmable</Typography>
                            <Typography color={"text.secondary"}>
                                Create custom scripts for your server in TypeScript using either the online editor or the vs code extension.<br />
                                We provide API's for storage, timers, scheduled tasks, configuration and more!
                            </Typography>
                        </Box>
                        <Box>
                            <Typography >No hosting, setup, installing required</Typography>
                            <Typography color={"text.secondary"}>
                                All you have to do is add the bot to your server and you're good to go! just create a script through the web interface or add a plugin to your server.
                            </Typography>
                        </Box>
                    </Grid2>
                    <Grid2 xs={2} display={"flex"} flexDirection="column">
                        <img src="/logo192.png" alt="zzz" style={{ borderRadius: "50%", objectFit: "contain", minWidth: 0 }} ></img>
                    </Grid2>
                </Grid2>
            </Paper>
            <Grid2 container>
                <img src={pogshowcase} alt="screenshot" style={{ borderRadius: "10px", marginTop: "20px", objectFit: "contain", minWidth: 0 }}></img>
            </Grid2>
            <div style={{ display: "flex", justifyContent: "center" }}>
                <BlLink to="/servers">Control panel</BlLink>
                <Button href="https://discord.gg/HJM3MqVBfw">Discord server</Button>
                <Button href="/docs/">Documentation</Button>
            </div>
        </Container>
        <Divider sx={{ marginTop: 1 }} />
        <Typography variant='h3' align='center'>Plugin Showcase</Typography>
        <ViewPlugins />
    </>
}