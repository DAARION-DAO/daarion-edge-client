import React, { useState, useEffect } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { InstallPage } from "./pages/InstallPage";
import { PilotOnboardingPage } from "./pages/PilotOnboardingPage";
import "./index.css";

// Lightweight path router — no react-router dependency needed
function Root() {
  const [path, setPath] = useState(window.location.pathname);

  useEffect(() => {
    const onPop = () => setPath(window.location.pathname);
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);

  if (path === "/install") return <InstallPage />;
  if (path === "/worker-pilot") return <PilotOnboardingPage />;
  return <App />;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);

