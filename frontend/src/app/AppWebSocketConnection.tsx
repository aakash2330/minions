import { useEffect } from "react";

import { registerMinionChatWebSocketHandlers } from "@/features/minion-chat/minionChatWebSocket";
import { appWebSocketClient } from "@/features/websocket/webSocketClient";

export function AppWebSocketConnection() {
  useEffect(() => {
    const unregisterMinionChatHandlers = registerMinionChatWebSocketHandlers();

    appWebSocketClient.connect();

    return () => {
      unregisterMinionChatHandlers();
      appWebSocketClient.disconnect();
    };
  }, []);

  return null;
}
