import React from 'react';
import './App.css';
import {
  BrowserRouter as Router,
} from "react-router-dom";
import './misc/WebsocketController';
import { RoutesElement } from './Routes';

function App() {
  return (
    <Router >
      <RoutesElement />
    </Router>
  );
}

export default App;

