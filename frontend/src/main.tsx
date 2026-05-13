import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter } from "react-router-dom";

import { App } from "@/app/App";
import { queryClient } from "@/app/queryClient";
import { WebsocketProvider } from "@/app/websocket/useWebsocket";
import "./styles.css";

createRoot(document.getElementById("app")!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <WebsocketProvider>
          <App />
        </WebsocketProvider>
      </BrowserRouter>
    </QueryClientProvider>
  </StrictMode>,
);
