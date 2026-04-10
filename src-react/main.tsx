import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import "./themes/amethyst-haze.css";
import "./themes/solar-dusk.css";
import "./themes/nature.css";
import { initializeTheme } from "./lib/theme";

// Initialize theme before rendering
initializeTheme();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
