import { create } from "zustand";

export enum WebSocketConnectionPhase {
  Connected = "connected",
  Connecting = "connecting",
  Disconnected = "disconnected",
  Error = "error",
  Idle = "idle",
}

export type WebSocketCloseDetails = {
  code: number;
  reason: string;
};

type WebSocketState = {
  closeDetails: WebSocketCloseDetails | null;
  lastError: string | null;
  phase: WebSocketConnectionPhase;
  socketUrl: string | null;
  recordClose: (details: WebSocketCloseDetails) => void;
  recordConnectionAttempt: (socketUrl: string) => void;
  recordError: (message: string) => void;
  recordOpen: () => void;
};

const initialState = {
  closeDetails: null,
  lastError: null,
  phase: WebSocketConnectionPhase.Idle,
  socketUrl: null,
} satisfies Omit<
  WebSocketState,
  "recordClose" | "recordConnectionAttempt" | "recordError" | "recordOpen"
>;

export const useWebSocketStore = create<WebSocketState>((set) => ({
  ...initialState,
  recordClose: (details) => {
    set({
      closeDetails: details,
      phase: WebSocketConnectionPhase.Disconnected,
    });
  },
  recordConnectionAttempt: (socketUrl) => {
    set({
      closeDetails: null,
      lastError: null,
      phase: WebSocketConnectionPhase.Connecting,
      socketUrl,
    });
  },
  recordError: (message) => {
    set({
      lastError: message,
      phase: WebSocketConnectionPhase.Error,
    });
  },
  recordOpen: () => {
    set({
      closeDetails: null,
      lastError: null,
      phase: WebSocketConnectionPhase.Connected,
    });
  },
}));
