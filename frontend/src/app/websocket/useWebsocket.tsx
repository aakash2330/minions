import {
  createContext,
  type PropsWithChildren,
  useContext,
  useEffect,
  useRef,
} from "react";

import {
  type ServerMessage,
  ServerMessageSchema,
} from "./serverMessage";

type WebsocketContextValue = {
  sendMessage: (message: ServerMessage) => void;
};

const WebsocketContext = createContext<WebsocketContextValue | null>(null);

export function WebsocketProvider({ children }: PropsWithChildren) {
  const socketRef = useRef<WebSocket | null>(null);

  if (!socketRef.current) {
    socketRef.current = new WebSocket(import.meta.env.VITE_WS_URL ?? "/ws");
  }

  function sendMessage(message: ServerMessage) {
    if (socketRef.current?.readyState !== WebSocket.OPEN) {
      throw new Error("Websocket is not open.");
    }

    const serverMessagePayload = ServerMessageSchema.parse(message);
    socketRef.current.send(JSON.stringify(serverMessagePayload));
  }

  useEffect(() => {
    return () => {
      socketRef.current?.close();
      socketRef.current = null;
    };
  }, []);

  return (
    <WebsocketContext.Provider value={{ sendMessage }}>
      {children}
    </WebsocketContext.Provider>
  );
}

export function useWebsocket() {
  const context = useContext(WebsocketContext);

  if (!context) {
    throw new Error("useWebsocket must be used within a WebsocketProvider.");
  }

  return context;
}
