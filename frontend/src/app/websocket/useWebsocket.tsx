import {
  createContext,
  type PropsWithChildren,
  useContext,
  useEffect,
  useRef,
} from "react";

import {
  type WebsocketClientMessage,
  WebsocketClientMessageSchema,
} from "./websocketMessages";

type WebsocketContextValue = {
  socket: WebSocket | null;
  sendMessage: (message: WebsocketClientMessage) => void;
};

const WebsocketContext = createContext<WebsocketContextValue | null>(null);

export function WebsocketProvider({ children }: PropsWithChildren) {
  const socketRef = useRef<WebSocket | null>(null);

  if (!socketRef.current) {
    socketRef.current = new WebSocket(import.meta.env.VITE_WS_URL ?? "/ws");
  }

  function sendMessage(message: WebsocketClientMessage) {
    if (socketRef.current?.readyState !== WebSocket.OPEN) {
      throw new Error("Websocket is not open.");
    }

    const apiMessage = WebsocketClientMessageSchema.parse(message);
    socketRef.current.send(JSON.stringify(apiMessage));
  }

  useEffect(() => {
    return () => {
      socketRef.current?.close();
      socketRef.current = null;
    };
  }, []);

  return (
    <WebsocketContext.Provider value={{ socket: socketRef.current, sendMessage }}>
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
