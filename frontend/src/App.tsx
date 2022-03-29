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
import pogshowcase from './img/pogshowcase.png';
import { TosPage } from './pages/TOS';
import { PrivacyPolicyPage } from './pages/PrivacyPolicy';
import './misc/WebsocketController';
import { Panel } from './components/Panel';
import { NewsPage } from './pages/NewsPage';
import { SamplesPage } from './pages/SamplesPage';

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
                  <NewsPage></NewsPage>
                </Route>
                <Route path="/samples">
                  <TopNav />
                  <div className="page-wrapper"><SamplesPage></SamplesPage></div>
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
      <p>Botloader<br></br>A programmable discord bot</p>
      <small>Verified and in beta!</small>
      <img src="/logo192.png" alt="zzz" className="avatar"></img>
      <div className='frontpage-links'>
        <Link to="/servers" className='bl-button' >Control panel</Link>
        <a className='bl-button' href="https://discord.gg/HJM3MqVBfw">Discord server</a>
        <a className='bl-button' href="/docs/">Documentation</a>
      </div>
    </header>
    <div className='frontpage-intro-container'>
      <div className='frontpage-intro'>
        <p>Botloader is a programmable discord bot that is currently in a beta phase.</p>
        <p>You can find more information using the links at the top and by joining the server.</p>
        <img src={pogshowcase} alt="screenshot"></img>
      </div>
    </div>
  </>
}

function TodoPage() {
  return <Panel>TODO</Panel>
}