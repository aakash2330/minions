import { Direction } from "./characters/characterConfig";

export enum MinionId {
  Kevin = "kevin",
  Bob = "bob",
}

export enum MinionElementKind {
  Workdesk = "workdesk",
}

export const MINION_ELEMENT_KINDS = Object.values(MinionElementKind);

type Point = {
  x: number;
  y: number;
};

export type MinionElementConfig = {
  id: string;
  label: string;
  position: Point;
  approach: Point & {
    facing: Direction;
  };
};

export type MinionMapConfig = {
  id: MinionId;
  name: string;
  spawn: Point & {
    facing: Direction;
  };
  elements: Partial<Record<MinionElementKind, MinionElementConfig>>;
};

export const MINION_MAP_CONFIGS: MinionMapConfig[] = [
  {
    id: MinionId.Kevin,
    name: "Kevin",
    spawn: { x: 234, y: 330, facing: Direction.Down },
    elements: {
      [MinionElementKind.Workdesk]: {
        id: "kevin-workdesk",
        label: "desk",
        position: { x: 206, y: 88 },
        approach: { x: 234, y: 133, facing: Direction.Up },
      },
    },
  },
  {
    id: MinionId.Bob,
    name: "Bob",
    spawn: { x: 702, y: 330, facing: Direction.Down },
    elements: {
      [MinionElementKind.Workdesk]: {
        id: "bob-workdesk",
        label: "desk",
        position: { x: 674, y: 88 },
        approach: { x: 702, y: 133, facing: Direction.Up },
      },
    },
  },
];

export function getMinionConfigById(minionId: MinionId) {
  return MINION_MAP_CONFIGS.find((config) => config.id === minionId);
}
