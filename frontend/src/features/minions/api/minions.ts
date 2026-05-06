import { z } from "zod";

import { Direction } from "@/game/characters/characterConfig";
import {
  getMapElementApproach,
  isMapElementKind,
  type MinionElementConfig,
  type MinionMapConfig,
  type Point,
  type PointWithFacing,
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
  session_id: z.string(),
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
  elements: z.array(ApiSessionElementSchema),
});

const ApiSessionsSchema = z.array(ApiSessionSchema);

type ApiPoint = z.infer<typeof ApiPointSchema>;
type ApiPointWithFacing = z.infer<typeof ApiPointWithFacingSchema>;
type ApiSessionElement = z.infer<typeof ApiSessionElementSchema>;
type ApiSession = z.infer<typeof ApiSessionSchema>;

export async function fetchMinions(): Promise<MinionMapConfig[]> {
  const response = await fetch("/api/sessions");

  if (!response.ok) {
    throw new Error(`Failed to load sessions: ${response.status}`);
  }

  const result = ApiSessionsSchema.safeParse(await response.json());

  if (!result.success) {
    throw new Error(
      `Invalid sessions response: ${formatZodError(result.error)}`,
    );
  }

  return result.data.map(toMinionMapConfig);
}

export function getMinionConfigBySessionId(
  minions: MinionMapConfig[],
  sessionId: string,
) {
  return minions.find((minion) => minion.sessionId === sessionId);
}

function toMinionMapConfig(minion: ApiSession): MinionMapConfig {
  return {
    sessionId: minion.session_id,
    workspaceId: minion.workspaceId,
    name: minion.name,
    kind: minion.kind,
    status: minion.status,
    spawn: toPointWithFacing(minion.spawn, Direction.Down),
    current: toPointWithFacing(minion.current, Direction.Down),
    elements: toMinionElementsByKind(minion.elements),
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
        sessionId: element.session_id,
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
