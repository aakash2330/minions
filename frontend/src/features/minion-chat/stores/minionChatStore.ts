import { create } from "zustand";

import type { SessionId } from "@/game/minionMapConfig";

export enum ChatSpeaker {
  Minion = "minion",
  Player = "player",
}

export type ChatMessage = {
  id: number;
  speaker: ChatSpeaker;
  text: string;
};

type MinionChatState = {
  draftMessagesBySessionId: Partial<Record<SessionId, string>>;
  isOpen: boolean;
  messagesBySessionId: Partial<Record<SessionId, ChatMessage[]>>;
  selectedSessionId: SessionId | null;
  openChat: (sessionId: SessionId) => void;
  sendPlayerMessage: () => void;
  setDraftMessage: (message: string) => void;
  setOpen: (isOpen: boolean) => void;
};

export const useMinionChatStore = create<MinionChatState>((set, get) => ({
  draftMessagesBySessionId: {},
  isOpen: false,
  messagesBySessionId: {},
  selectedSessionId: null,
  openChat: (sessionId) => {
    set({
      isOpen: true,
      selectedSessionId: sessionId,
    });
  },
  sendPlayerMessage: () => {
    const { draftMessagesBySessionId, selectedSessionId } = get();

    if (!selectedSessionId) {
      return;
    }

    const trimmedText = (
      draftMessagesBySessionId[selectedSessionId] ?? ""
    ).trim();

    if (!trimmedText) {
      return;
    }

    set((state) => ({
      draftMessagesBySessionId: {
        ...state.draftMessagesBySessionId,
        [selectedSessionId]: "",
      },
      messagesBySessionId: {
        ...state.messagesBySessionId,
        [selectedSessionId]: [
          ...(state.messagesBySessionId[selectedSessionId] ?? []),
          {
            id: Date.now(),
            speaker: ChatSpeaker.Player,
            text: trimmedText,
          },
        ],
      },
    }));
  },
  setDraftMessage: (message) => {
    const { selectedSessionId } = get();

    if (!selectedSessionId) {
      return;
    }

    set((state) => ({
      draftMessagesBySessionId: {
        ...state.draftMessagesBySessionId,
        [selectedSessionId]: message,
      },
    }));
  },
  setOpen: (isOpen) => {
    set({ isOpen });
  },
}));
