import { Direction } from "@/game/characters/characterConfig";
import {
  getMapElementApproach,
  isMapElementKind,
  type SessionElementConfig,
  type SessionMapConfig,
  type Point,
  type PointWithFacing,
} from "@/game/sessionMapConfig";

import type {
  ApiSession,
  ApiSessionElement,
  ApiSessionMessage,
  ApiPoint,
  ApiPointWithFacing,
  ApiWorkspace,
} from "./sessionSchemas";

export enum SessionMessageRole {
  Assistant = "assistant",
  System = "system",
  User = "user",
}

export type SessionMessage = {
  id: string;
  sessionId: string;
  role: SessionMessageRole | string;
  status: string;
  text: string;
};

export type Session = SessionMapConfig & {
  messages: SessionMessage[];
};

export function getAssignedElementsBySessionId(
  elementsByWorkspaceId: Map<string, ApiSessionElement[]>,
) {
  const elementsBySessionId = new Map<string, ApiSessionElement[]>();

  for (const elements of elementsByWorkspaceId.values()) {
    for (const element of elements) {
      if (!element.assignedSessionId) {
        continue;
      }

      const sessionElements =
        elementsBySessionId.get(element.assignedSessionId) ?? [];
      sessionElements.push(element);
      elementsBySessionId.set(element.assignedSessionId, sessionElements);
    }
  }

  return elementsBySessionId;
}

export function toSession(
  session: ApiSession,
  workspace: ApiWorkspace | undefined,
  elements: ApiSessionElement[],
): Session {
  return {
    sessionId: session.sessionId,
    workspaceId: session.workspaceId,
    workspaceRootPath: workspace?.rootPath ?? null,
    name: session.name,
    kind: session.kind,
    status: session.status,
    spawn: toPointWithFacing(session.spawn, Direction.Down),
    current: toPointWithFacing(session.current, Direction.Down),
    elements: toSessionElementsByKind(elements),
    messages: session.messages.map(toSessionMessage),
  };
}

function toSessionMessage(message: ApiSessionMessage): SessionMessage {
  return {
    id: message.id,
    sessionId: message.sessionId,
    role: toSessionMessageRole(message.role),
    status: message.status,
    text: message.text,
  };
}

function toSessionMessageRole(role: string) {
  if (Object.values(SessionMessageRole).includes(role as SessionMessageRole)) {
    return role as SessionMessageRole;
  }

  return role;
}

function toSessionElementsByKind(elements: ApiSessionElement[]) {
  return elements.reduce<SessionMapConfig["elements"]>(
    (elementsByKind, element) => {
      if (!isMapElementKind(element.kind)) {
        return elementsByKind;
      }

      const position = toPoint(element.position);
      const facing = toDirection(element.facing, Direction.Up);
      const sessionElement: SessionElementConfig = {
        id: element.id,
        kind: element.kind,
        label: element.label,
        position,
        approach: getMapElementApproach(element.kind, position, facing),
      };

      return {
        ...elementsByKind,
        [element.kind]: sessionElement,
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
