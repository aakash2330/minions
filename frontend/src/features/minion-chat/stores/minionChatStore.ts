import type { SessionId } from "@/game/minionMapConfig";
import { create } from "zustand";

export enum ChatSpeaker {
  Minion = "minion",
  Player = "player",
}

type MinionChatState = {
  isOpen: boolean;
  selectedSessionId: SessionId | null;
  openChat: (sessionId: SessionId) => void;
  setOpen: (isOpen: boolean) => void;
};

export const useMinionChatStore = create<MinionChatState>((set, get) => ({
  isOpen: false,
  selectedSessionId: null,
  openChat: (sessionId) => {
    set({
      isOpen: true,
      selectedSessionId: sessionId,
    });
  },
  setOpen: (isOpen) => {
    set({
      isOpen,
      selectedSessionId: isOpen ? get().selectedSessionId : null,
    });
  },
}));
