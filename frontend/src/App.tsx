import React from 'react';
import './App.css';
import {
  createBrowserRouter,
  RouterProvider,
} from "react-router-dom";
import './modules/websocket/WebsocketController';
import { routes } from './pages';

import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

import { createTheme, ThemeProvider } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';
import { SideNavStateController } from './components/SideNavManager';
import { Notifications } from './components/Notifications';
import { SessionProvider } from './modules/session/SessionContext';
import { GuildsProvider } from './modules/guilds/GuildsProvider';

const darkTheme = createTheme({
  palette: {
    mode: 'dark',
  },
});

const router = createBrowserRouter(routes)

function App() {
  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <Notifications>
        <SideNavStateController>
          <SessionProvider>
            <GuildsProvider>
              <RouterProvider router={router} />
            </GuildsProvider>
          </SessionProvider>
        </SideNavStateController>
      </Notifications>
    </ThemeProvider>
  );
}

export default App;

