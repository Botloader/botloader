import { Button, Container, Paper } from '@mui/material';
import Grid2 from '@mui/material/Unstable_Grid2/Grid2';
import { Link } from 'react-router-dom';
import pogshowcase from '../img/pogshowcase.png';

export function LandingPage() {
    return <>
        <Container>
            <p style={{ backgroundColor: "#378855", borderRadius: "10px", padding: "10px", textAlign: "center" }}>Verified and in beta!</p>
            <Paper>
                <Grid2 container spacing={2} padding={2}>
                    <Grid2 xs={10} display={"flex"} justifyContent="center" flexDirection={"column"}>
                        <p>Create custom bots for your discord servers in minutes without having to install or host anything!</p>
                        <p>Program <b>TypeScript</b> scripts for your server in a online code editor, the same code editor inside in visual studio code!</p>
                        <p>We provide API's for storage, timers, scheduled tasks and more to come!</p>
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
                <Link to="/servers"><Button>Control panel</Button></Link>
                <Button href="https://discord.gg/HJM3MqVBfw">Discord server</Button>
                <Button href="/docs/">Documentation</Button>
            </div>
        </Container>
    </>
}