import {
  getMapElementApproach,
  type SessionElementConfig,
  type SessionMapConfig,
  type Point,
  type PointWithFacing,
} from "@/game/sessionMapConfig";

import {
  ApiSessionMessageStatus,
  type ApiPoint,
  type ApiPointWithFacing,
  type ApiSession,
  type ApiSessionElement,
  type ApiSessionMessage,
  type ApiWorkspace,
} from "./sessionSchemas";

export enum SessionMessageRole {
  Assistant = "assistant",
  System = "system",
  User = "user",
}

export enum SessionMessageStatus {
  Pending = "pending",
  Streaming = "streaming",
  Complete = "complete",
  Error = "error",
}

export type SessionMessage = {
  id: string;
  sessionId: string;
  role: SessionMessageRole | string;
  status: SessionMessageStatus;
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
    spawn: toPointWithFacing(session.spawn),
    current: toPointWithFacing(session.current),
    elements: toSessionElementsByKind(elements),
    messages: session.messages.map(toSessionMessage),
  };
}

function toSessionMessage(message: ApiSessionMessage): SessionMessage {
  return {
    id: message.id,
    sessionId: message.sessionId,
    role: toSessionMessageRole(message.role),
    status: toSessionMessageStatus(message.status),
    text: message.text,
  };
}

function toSessionMessageRole(role: string) {
  if (Object.values(SessionMessageRole).includes(role as SessionMessageRole)) {
    return role as SessionMessageRole;
  }

  return role;
}

function toSessionMessageStatus(
  status: ApiSessionMessage["status"],
): SessionMessageStatus {
  switch (status) {
    case ApiSessionMessageStatus.Pending:
      return SessionMessageStatus.Pending;
    case ApiSessionMessageStatus.Streaming:
      return SessionMessageStatus.Streaming;
    case ApiSessionMessageStatus.Complete:
      return SessionMessageStatus.Complete;
    case ApiSessionMessageStatus.Error:
      return SessionMessageStatus.Error;
  }
}

function toSessionElementsByKind(elements: ApiSessionElement[]) {
  return elements.reduce<SessionMapConfig["elements"]>(
    (elementsByKind, element) => {
      const position = toPoint(element.position);
      const sessionElement: SessionElementConfig = {
        id: element.id,
        kind: element.kind,
        label: element.label,
        position,
        approach: getMapElementApproach(element.kind, position, element.facing),
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

function toPointWithFacing(point: ApiPointWithFacing): PointWithFacing {
  return {
    ...toPoint(point),
    facing: point.facing,
  };
}
