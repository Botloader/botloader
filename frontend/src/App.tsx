import React from 'react';
import './App.css';
import {
  createBrowserRouter,
  BrowserRouter as Router,
  RouterProvider,
} from "react-router-dom";
import './misc/WebsocketController';
import { routes } from './pages';

import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

import { createTheme, ThemeProvider } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';
import { SideNavStateController } from './components/SideNavManager';
import { Notifications } from './components/Notifications';

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
          <RouterProvider router={router} />
          {/* <Router >
                    </Router> */}

        </SideNavStateController>
      </Notifications>
    </ThemeProvider>
  );
}

export default App;

