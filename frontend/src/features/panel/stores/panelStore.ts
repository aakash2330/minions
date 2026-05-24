import { create } from "zustand";

export enum PanelContentType {
  GlobalChat = "global-chat",
  SessionChat = "session-chat",
}

export type PanelContent =
  | {
      type: PanelContentType.GlobalChat;
      workspaceId: string;
    }
  | {
      type: PanelContentType.SessionChat;
      sessionId: string;
    };

type PanelState = {
  isOpen: boolean;
  content: PanelContent | null;
  open: (content: PanelContent) => void;
  close: () => void;
  setOpen: (isOpen: boolean) => void;
};

export const usePanelStore = create<PanelState>((set, get) => ({
  isOpen: false,
  content: null,
  open: (content) => {
    set({
      isOpen: true,
      content,
    });
  },
  close: () => {
    set({
      isOpen: false,
      content: null,
    });
  },
  setOpen: (isOpen) => {
    set({
      isOpen,
      content: isOpen ? get().content : null,
    });
  },
}));
