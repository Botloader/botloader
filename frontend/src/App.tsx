import React from 'react';
import './App.css';
import {
  BrowserRouter as Router,
  Switch,
  Route,
  useParams,
  Link,
} from "react-router-dom";
import { RequireLoggedInSession, SessionProvider } from './components/Session';
import { CurrentGuildProvider, GuildsProvider } from './components/GuildsProvider';
import { TopNav } from './components/TopNav';
import { ConfirmLoginPage } from './pages/ConfirmLogin';
import { SelectServerPage } from './pages/SelectServer';
import { UserSettingsPage } from './pages/UserSettings';
import { GuildPage } from './pages/GuildPage';
import loaderScreenshot from './img/loaderscreenshot.png';
import { TosPage } from './pages/TOS';
import { PrivacyPolicyPage } from './pages/PrivacyPolicy';
import './misc/WebsocketController';
import { Panel } from './components/Panel';

function App() {
  return (
    <Router>
      <Switch>
        <Route path="/confirm_login">
          <ConfirmLoginPage />
        </Route>
        <Route path="/tos">
          <TosPage></TosPage>
        </Route>
        <Route path="/privacy">
          <PrivacyPolicyPage></PrivacyPolicyPage>
        </Route>
        <Route path="/">
          <SessionProvider>
            <GuildsProvider>
              <Switch>
                <Route path="/news">
                  <TopNav />
                  <TodoPage></TodoPage>
                </Route>
                <Route path="/samples">
                  <TopNav />
                  <TodoPage></TodoPage>
                </Route>
                <Route path="/premium">
                  <TopNav />
                  <TodoPage></TodoPage>
                </Route>
                <Route path="/settings">
                  <TopNav />
                  <RequireLoggedInSession>
                    <div className="page-wrapper"><UserSettingsPage /></div>
                  </RequireLoggedInSession>
                </Route>
                <Route path="/servers">
                  <Switch>
                    <Route path="/servers/:guildId">
                      <RequireLoggedInSession>
                        <OuterGuildPage />
                      </RequireLoggedInSession>
                    </Route>
                    <Route path="/servers">
                      <TopNav />
                      <div className="page-wrapper"><SelectServerPage /></div>
                    </Route>
                  </Switch>
                </Route>
                <Route path="/">
                  <TopNav />
                  <LandingPage />
                </Route>
              </Switch>
            </GuildsProvider>
          </SessionProvider>
        </Route>
      </Switch>
    </Router>
  );
}

function OuterGuildPage() {
  let { guildId }: { guildId: string } = useParams();

  return <CurrentGuildProvider guildId={guildId}>
    <TopNav />
    <GuildPage />
  </CurrentGuildProvider>
}

export default App;


function LandingPage() {
  return <>
    <header className="App-header">
      <p>Botloader coming soon™</p>
      <img src="/logo192.png" alt="zzz" className="avatar"></img>
      <div className='frontpage-links'>
        <Link to="/servers" className='bl-button' >Control panel</Link>
        <a className='bl-button' href="https://discord.gg/HJM3MqVBfw">Discord server</a>
        <a className='bl-button' href="/docs/">Documentation</a>
      </div>
    </header>
    <div className='frontpage-intro-container'>
      <div className='frontpage-intro'>
        <p>
          <b>Botloader is a programmable discord bot that is currently in a early private alpha phase.</b><br />
          <br />This project has 2 goals
          <br />The first goal is to be a platform for server admins to create server specific scripts, tools, games etc.
          <br />How it works is you add botloader to your server, then you can instantly start programming it.<br />
          <br />The second goal is to provide a marketplace of user created "plugins" that you can add to your server with a single click.<br />
          <br />The project is currently in a early private alpha phase where were focusing on adding all the scripting API's and making sure that the programming experience is ergonomic and other fancy ass words.
        </p>
        <img src={loaderScreenshot} alt="screenshot"></img>
      </div>
    </div>
  </>
}

function TodoPage() {
  return <Panel>TODO</Panel>
}