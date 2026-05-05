import { Direction } from "@/game/characters/characterConfig";
import {
  getMapElementApproach,
  isMapElementKind,
  type MinionElementConfig,
  type MinionMapConfig,
  type Point,
  type PointWithFacing,
} from "@/game/minionMapConfig";

type ApiPoint = {
  x: number;
  y: number;
};

type ApiPointWithFacing = ApiPoint & {
  facing: string;
};

type ApiMinionElement = {
  id: string;
  minionId: string;
  kind: string;
  label: string;
  position: ApiPoint;
  facing: string;
};

type ApiMinion = {
  id: string;
  workspaceId: string;
  name: string;
  kind: string;
  status: string;
  spawn: ApiPointWithFacing;
  current: ApiPointWithFacing;
  elements: ApiMinionElement[];
};

export async function fetchMinions(): Promise<MinionMapConfig[]> {
  const response = await fetch("/api/minions");

  if (!response.ok) {
    throw new Error(`Failed to load minions: ${response.status}`);
  }

  const minions = (await response.json()) as ApiMinion[];

  return minions.map(toMinionMapConfig);
}

export function getMinionConfigById(
  minions: MinionMapConfig[],
  minionId: string,
) {
  return minions.find((minion) => minion.id === minionId);
}

function toMinionMapConfig(minion: ApiMinion): MinionMapConfig {
  return {
    id: minion.id,
    workspaceId: minion.workspaceId,
    name: minion.name,
    kind: minion.kind,
    status: minion.status,
    spawn: toPointWithFacing(minion.spawn, Direction.Down),
    current: toPointWithFacing(minion.current, Direction.Down),
    elements: toMinionElementsByKind(minion.elements),
  };
}

function toMinionElementsByKind(elements: ApiMinionElement[]) {
  return elements.reduce<MinionMapConfig["elements"]>(
    (elementsByKind, element) => {
      if (!isMapElementKind(element.kind)) {
        return elementsByKind;
      }

      const position = toPoint(element.position);
      const facing = toDirection(element.facing, Direction.Up);
      const minionElement: MinionElementConfig = {
        id: element.id,
        kind: element.kind,
        label: element.label,
        minionId: element.minionId,
        position,
        approach: getMapElementApproach(element.kind, position, facing),
      };

      return {
        ...elementsByKind,
        [element.kind]: minionElement,
      };
    },
    {},
  );
}

function toPoint(point: ApiPoint): Point {
  return {
    x: point.x,
    y: point.y,
  };
}

function toPointWithFacing(
  point: ApiPointWithFacing,
  fallbackDirection: Direction,
): PointWithFacing {
  return {
    ...toPoint(point),
    facing: toDirection(point.facing, fallbackDirection),
  };
}

function toDirection(value: string, fallbackDirection: Direction) {
  if (Object.values(Direction).includes(value as Direction)) {
    return value as Direction;
  }

  return fallbackDirection;
}
