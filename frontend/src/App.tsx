import React from 'react';
import './App.css';
import {
  BrowserRouter as Router,
} from "react-router-dom";
import './misc/WebsocketController';
import { RoutesElement } from './Routes';

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

function App() {
  return (
    <ThemeProvider theme={darkTheme}>
      <CssBaseline />
      <Notifications>
        <SideNavStateController>
          <Router >
            <RoutesElement />
          </Router>
        </SideNavStateController>
      </Notifications>
    </ThemeProvider>
  );
}

export default App;

