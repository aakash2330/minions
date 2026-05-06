import {
  parseServerEvent,
  serializeClientMessage,
  type ClientMessage,
  type ServerEvent,
  type ServerEventByType,
  type ServerEventType,
} from "./protocol";
import { useWebSocketStore } from "./webSocketStore";

type ServerEventHandler<TType extends ServerEventType> = (
  event: ServerEventByType<TType>,
) => void;

type WebSocketUrlProvider = string | (() => string);

export class AppWebSocketClient {
  private readonly handlersByType = new Map<
    ServerEventType,
    Set<(event: ServerEvent) => void>
  >();
  private socket: WebSocket | null = null;
  private readonly urlProvider: WebSocketUrlProvider;

  constructor(urlProvider: WebSocketUrlProvider = getDefaultWebSocketUrl) {
    this.urlProvider = urlProvider;
  }

  connect() {
    if (
      this.socket?.readyState === WebSocket.OPEN ||
      this.socket?.readyState === WebSocket.CONNECTING
    ) {
      return;
    }

    const socketUrl = this.resolveSocketUrl();
    useWebSocketStore.getState().recordConnectionAttempt(socketUrl);

    try {
      const socket = new WebSocket(socketUrl);
      this.socket = socket;

      socket.addEventListener("open", () => {
        if (socket !== this.socket) {
          return;
        }

        useWebSocketStore.getState().recordOpen();
      });

      socket.addEventListener("message", (event) => {
        if (typeof event.data !== "string") {
          return;
        }

        this.handleMessage(event.data);
      });

      socket.addEventListener("error", () => {
        useWebSocketStore
          .getState()
          .recordError("Unable to connect to the backend WebSocket.");
      });

      socket.addEventListener("close", (event) => {
        if (socket !== this.socket) {
          return;
        }

        this.socket = null;
        useWebSocketStore.getState().recordClose({
          code: event.code,
          reason: event.reason,
        });
      });
    } catch (error) {
      useWebSocketStore.getState().recordError(formatErrorMessage(error));
    }
  }

  disconnect() {
    const socket = this.socket;
    this.socket = null;

    if (
      socket &&
      socket.readyState !== WebSocket.CLOSED &&
      socket.readyState !== WebSocket.CLOSING
    ) {
      socket.close();
    }
  }

  send(message: ClientMessage) {
    if (this.socket?.readyState !== WebSocket.OPEN) {
      useWebSocketStore.getState().recordError("WebSocket is not connected.");
      return false;
    }

    this.socket.send(JSON.stringify(serializeClientMessage(message)));
    return true;
  }

  on<TType extends ServerEventType>(
    type: TType,
    handler: ServerEventHandler<TType>,
  ) {
    const handlers = this.handlersByType.get(type) ?? new Set();
    const untypedHandler = handler as (event: ServerEvent) => void;
    handlers.add(untypedHandler);
    this.handlersByType.set(type, handlers);

    return () => {
      handlers.delete(untypedHandler);
      if (handlers.size === 0) {
        this.handlersByType.delete(type);
      }
    };
  }

  private handleMessage(rawMessage: string) {
    const event = parseServerEvent(rawMessage);

    if (!event) {
      useWebSocketStore
        .getState()
        .recordError("Received an invalid WebSocket event.");
      return;
    }

    this.emit(event);
  }

  private emit(event: ServerEvent) {
    for (const handler of this.handlersByType.get(event.type) ?? []) {
      handler(event);
    }
  }

  private resolveSocketUrl() {
    const rawUrl =
      typeof this.urlProvider === "function"
        ? this.urlProvider()
        : this.urlProvider;

    return resolveWebSocketUrl(rawUrl);
  }
}

export const appWebSocketClient = new AppWebSocketClient();

function getDefaultWebSocketUrl() {
  const configuredUrl = import.meta.env.VITE_WS_URL as string | undefined;

  if (configuredUrl?.trim()) {
    return configuredUrl;
  }

  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";

  return `${protocol}//${window.location.host}/ws`;
}

function resolveWebSocketUrl(rawUrl: string) {
  const url = new URL(rawUrl, window.location.href);

  if (url.protocol === "http:") {
    url.protocol = "ws:";
  } else if (url.protocol === "https:") {
    url.protocol = "wss:";
  }

  if (url.protocol !== "ws:" && url.protocol !== "wss:") {
    throw new Error(`Unsupported WebSocket URL protocol: ${url.protocol}`);
  }

  url.pathname = "/ws";

  return url.toString();
}

function formatErrorMessage(error: unknown) {
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  return String(error);
}
