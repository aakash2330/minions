import { Direction } from "@/game/characters/characterConfig";
import {
  getMapElementApproach,
  isMapElementKind,
  type MinionElementConfig,
  type MinionMapConfig,
  type Point,
  type PointWithFacing,
} from "@/game/minionMapConfig";

import type {
  ApiMinion,
  ApiMinionElement,
  ApiMinionMessage,
  ApiPoint,
  ApiPointWithFacing,
  ApiWorkspace,
} from "./minionSchemas";

export enum MinionMessageRole {
  Assistant = "assistant",
  System = "system",
  User = "user",
}

export type MinionMessage = {
  id: string;
  minionId: string;
  role: MinionMessageRole | string;
  status: string;
  text: string;
};

export type Minion = MinionMapConfig & {
  messages: MinionMessage[];
};

export function getAssignedElementsByMinionId(
  elementsByWorkspaceId: Map<string, ApiMinionElement[]>,
) {
  const elementsByMinionId = new Map<string, ApiMinionElement[]>();

  for (const elements of elementsByWorkspaceId.values()) {
    for (const element of elements) {
      if (!element.assignedMinionId) {
        continue;
      }

      const minionElements =
        elementsByMinionId.get(element.assignedMinionId) ?? [];
      minionElements.push(element);
      elementsByMinionId.set(element.assignedMinionId, minionElements);
    }
  }

  return elementsByMinionId;
}

export function toMinion(
  minion: ApiMinion,
  workspace: ApiWorkspace | undefined,
  elements: ApiMinionElement[],
): Minion {
  return {
    minionId: minion.minionId,
    workspaceId: minion.workspaceId,
    workspaceRootPath: workspace?.rootPath ?? null,
    name: minion.name,
    kind: minion.kind,
    status: minion.status,
    spawn: toPointWithFacing(minion.spawn, Direction.Down),
    current: toPointWithFacing(minion.current, Direction.Down),
    elements: toMinionElementsByKind(elements),
    messages: minion.messages.map(toMinionMessage),
  };
}

function toMinionMessage(message: ApiMinionMessage): MinionMessage {
  return {
    id: message.id,
    minionId: message.minionId,
    role: toMinionMessageRole(message.role),
    status: message.status,
    text: message.text,
  };
}

function toMinionMessageRole(role: string) {
  if (Object.values(MinionMessageRole).includes(role as MinionMessageRole)) {
    return role as MinionMessageRole;
  }

  return role;
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
