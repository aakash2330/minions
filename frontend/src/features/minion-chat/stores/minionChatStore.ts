import { create } from "zustand";

import type { MinionId } from "@/game/minionMapConfig";

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
  draftMessagesByMinionId: Partial<Record<MinionId, string>>;
  isOpen: boolean;
  messagesByMinionId: Partial<Record<MinionId, ChatMessage[]>>;
  selectedMinionId: MinionId | null;
  openChat: (minionId: MinionId) => void;
  sendPlayerMessage: () => void;
  setDraftMessage: (message: string) => void;
  setOpen: (isOpen: boolean) => void;
};

export const useMinionChatStore = create<MinionChatState>((set, get) => ({
  draftMessagesByMinionId: {},
  isOpen: false,
  messagesByMinionId: {},
  selectedMinionId: null,
  openChat: (minionId) => {
    set({
      isOpen: true,
      selectedMinionId: minionId,
    });
  },
  sendPlayerMessage: () => {
    const { draftMessagesByMinionId, selectedMinionId } = get();

    if (!selectedMinionId) {
      return;
    }

    const trimmedText = (
      draftMessagesByMinionId[selectedMinionId] ?? ""
    ).trim();

    if (!trimmedText) {
      return;
    }

    set((state) => ({
      draftMessagesByMinionId: {
        ...state.draftMessagesByMinionId,
        [selectedMinionId]: "",
      },
      messagesByMinionId: {
        ...state.messagesByMinionId,
        [selectedMinionId]: [
          ...(state.messagesByMinionId[selectedMinionId] ?? []),
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
    const { selectedMinionId } = get();

    if (!selectedMinionId) {
      return;
    }

    set((state) => ({
      draftMessagesByMinionId: {
        ...state.draftMessagesByMinionId,
        [selectedMinionId]: message,
      },
    }));
  },
  setOpen: (isOpen) => {
    set({ isOpen });
  },
}));
