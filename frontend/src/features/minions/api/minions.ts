import { z } from "zod";

import { Direction } from "@/game/characters/characterConfig";
import {
  getMapElementApproach,
  isMapElementKind,
  type MinionElementConfig,
  type MinionMapConfig,
  type Point,
  type PointWithFacing,
  type SessionId,
} from "@/game/minionMapConfig";
import { formatZodError } from "@/lib/zodError";

const ApiPointSchema = z.object({
  x: z.number(),
  y: z.number(),
});

const ApiPointWithFacingSchema = ApiPointSchema.extend({
  facing: z.string(),
});

const ApiSessionElementSchema = z.object({
  id: z.string(),
  assignedSessionId: z.string().nullable(),
  kind: z.string(),
  label: z.string(),
  position: ApiPointSchema,
  facing: z.string(),
});

const ApiSessionSchema = z.object({
  session_id: z.string(),
  workspaceId: z.string(),
  name: z.string(),
  kind: z.string(),
  status: z.string(),
  spawn: ApiPointWithFacingSchema,
  current: ApiPointWithFacingSchema,
});

const ApiWorkspaceSchema = z.object({
  id: z.string(),
  name: z.string(),
  rootPath: z.string().nullable(),
});

const ApiDataResponseSchema = z.object({
  workspaces: z.array(ApiWorkspaceSchema),
  sessions: z.array(ApiSessionSchema),
});

const ApiWorkspaceElementsSchema = z.array(ApiSessionElementSchema);

type ApiPoint = z.infer<typeof ApiPointSchema>;
type ApiPointWithFacing = z.infer<typeof ApiPointWithFacingSchema>;
type ApiSessionElement = z.infer<typeof ApiSessionElementSchema>;
type ApiSession = z.infer<typeof ApiSessionSchema>;
type ApiWorkspace = z.infer<typeof ApiWorkspaceSchema>;

export async function fetchMinions(): Promise<MinionMapConfig[]> {
  const response = await fetch("/api/data");

  if (!response.ok) {
    throw new Error(`Failed to load app data: ${response.status}`);
  }

  const result = ApiDataResponseSchema.safeParse(await response.json());

  if (!result.success) {
    throw new Error(
      `Invalid app data response: ${formatZodError(result.error)}`,
    );
  }

  const workspaceElements = await fetchElementsByWorkspaceId(
    result.data.workspaces,
  );
  const workspaceById = new Map(
    result.data.workspaces.map((workspace) => [workspace.id, workspace]),
  );
  const elementsBySessionId =
    getAssignedElementsBySessionId(workspaceElements);

  return result.data.sessions.map((session) =>
    toMinionMapConfig(
      session,
      workspaceById.get(session.workspaceId),
      elementsBySessionId.get(session.session_id) ?? [],
    ),
  );
}

export function getMinionConfigBySessionId(
  minions: MinionMapConfig[],
  sessionId: string,
) {
  return minions.find((minion) => minion.sessionId === sessionId);
}

async function fetchElementsByWorkspaceId(workspaces: ApiWorkspace[]) {
  const entries = await Promise.all(
    workspaces.map(async (workspace) => {
      const response = await fetch(
        `/api/workspaces/${encodeURIComponent(workspace.id)}/elements`,
      );

      if (!response.ok) {
        throw new Error(
          `Failed to load workspace elements: ${response.status}`,
        );
      }

      const result = ApiWorkspaceElementsSchema.safeParse(
        await response.json(),
      );

      if (!result.success) {
        throw new Error(
          `Invalid workspace elements response: ${formatZodError(result.error)}`,
        );
      }

      return [workspace.id, result.data] as const;
    }),
  );

  return new Map(entries);
}

function getAssignedElementsBySessionId(
  elementsByWorkspaceId: Map<string, ApiSessionElement[]>,
) {
  const elementsBySessionId = new Map<SessionId, ApiSessionElement[]>();

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

function toMinionMapConfig(
  minion: ApiSession,
  workspace: ApiWorkspace | undefined,
  elements: ApiSessionElement[],
): MinionMapConfig {
  return {
    sessionId: minion.session_id,
    workspaceId: minion.workspaceId,
    workspaceRootPath: workspace?.rootPath ?? null,
    name: minion.name,
    kind: minion.kind,
    status: minion.status,
    spawn: toPointWithFacing(minion.spawn, Direction.Down),
    current: toPointWithFacing(minion.current, Direction.Down),
    elements: toMinionElementsByKind(elements),
  };
}

function toMinionElementsByKind(elements: ApiSessionElement[]) {
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
