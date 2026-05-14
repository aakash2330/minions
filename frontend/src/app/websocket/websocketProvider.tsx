import {
  createContext,
  type PropsWithChildren,
  useContext,
  useEffect,
  useRef,
} from "react";

import { useHandleWsEvents } from "./events/useHandleWsEvents";
import { type WsMessage, WsMessageSchema } from "./messages/wsMessage";
import { useHandleWsMessage } from "./messages/useHandleWsMessage";

type WebsocketContextValue = {
  sendWsMessage: (message: WsMessage) => void;
};

const WebsocketContext = createContext<WebsocketContextValue | null>(null);

export function WebsocketProvider({ children }: PropsWithChildren) {
  const socketRef = useRef<WebSocket | null>(null);
  const { handleWsEvent } = useHandleWsEvents();
  const { handleWsMessage } = useHandleWsMessage();

  function sendWsMessage(message: WsMessage) {
    if (socketRef.current?.readyState !== WebSocket.OPEN) {
      throw new Error("Websocket is not open.");
    }
    let wsMessage = handleWsMessage(message);
    const paresedWsMessagePayload = WsMessageSchema.safeParse(wsMessage);
    if (!paresedWsMessagePayload.success) {
      throw new Error("failed to parse ws message");
    }
    socketRef.current.send(JSON.stringify(paresedWsMessagePayload.data));
  }

  useEffect(() => {
    if (!socketRef.current) {
      socketRef.current = new WebSocket(import.meta.env.VITE_WS_URL ?? "/ws");
      socketRef.current.addEventListener("message", handleWsEvent);
    }
    return () => {
      socketRef.current?.removeEventListener("message", handleWsEvent);
      socketRef.current?.close();
      socketRef.current = null;
    };
  }, [handleWsEvent]);

  return (
    <WebsocketContext.Provider value={{ sendWsMessage }}>
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
