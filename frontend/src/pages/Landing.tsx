import { Link } from 'react-router-dom';
import pogshowcase from '../img/pogshowcase.png';

export function LandingPage() {
    return <>
        <header className='intro-container'>
            <div className='intro-left' style={{ flexShrink: 1, maxWidth: "500px", marginRight: "10px" }}>
                <p style={{ backgroundColor: "#378855", borderRadius: "10px", padding: "10px", fontSize: "1.5rem" }}>Verified and in beta!</p>
                <section className="App-header">
                    <img src="/logo192.png" alt="zzz" className="avatar"></img>
                </section>
                <div className='frontpage-intro'>
                    <p>Create custom bots for your discord servers in minutes without having to install or host anything!</p>
                    <p>Program <b>TypeScript</b> scripts for your server in a online code editor, the same code editor inside in visual studio code!</p>
                    <p>We provide API's for storage, timers, scheduled tasks and more to come!</p>
                </div>
            </div>
            <div style={{ flexGrow: 1, display: "flex", justifyContent: "center", minWidth: 0 }}>
                <img className='screenshot' src={pogshowcase} alt="screenshot" style={{ borderRadius: "10px", marginTop: "20px", objectFit: "contain", minWidth: 0 }}></img>
            </div>
        </header>
        <div className='frontpage-links'>
            <Link to="/servers" className='bl-button' >Control panel</Link>
            <a className='bl-button' href="https://discord.gg/HJM3MqVBfw">Discord server</a>
            <a className='bl-button' href="/docs/">Documentation</a>
        </div>
    </>
}