import { create } from "zustand";

export enum AppSection {
  Conversations = "conversations",
  World = "world",
  Minions = "minions",
  Tasks = "tasks",
  Settings = "settings",
}

type AppNavigationState = {
  activeSection: AppSection;
  setActiveSection: (section: AppSection) => void;
};

export const useAppNavigationStore = create<AppNavigationState>((set) => ({
  activeSection: AppSection.World,
  setActiveSection: (section) => {
    set({ activeSection: section });
  },
}));
