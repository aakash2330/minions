import { useEffect } from "react";

import { appWebSocketClient } from "@/features/websocket/webSocketClient";

export function AppWebSocketConnection() {
  useEffect(() => {
    appWebSocketClient.connect();

    return () => {
      appWebSocketClient.disconnect();
    };
  }, []);

  return null;
}
