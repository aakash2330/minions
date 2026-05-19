import { Direction } from "./characters/characterConfig";
import type { WorkspaceElementKind } from "./workspaceElementKind";

export type Point = {
  x: number;
  y: number;
};

export type PointWithFacing = Point & {
  facing: Direction;
};

export type SessionElementConfig = {
  id: string;
  kind: WorkspaceElementKind;
  label: string;
  position: Point;
  approach: PointWithFacing;
};

export type StaticMapElementConfig = {
  id: string;
  kind: WorkspaceElementKind;
  label: string;
  position: Point;
};

export type SessionMapConfig = {
  sessionId: string;
  workspaceId: string;
  workspaceRootPath: string | null;
  name: string;
  kind: string;
  status: string;
  spawn: PointWithFacing;
  current: PointWithFacing;
  elements: Partial<Record<WorkspaceElementKind, SessionElementConfig>>;
};

export const STATIC_MAP_ELEMENTS: StaticMapElementConfig[] = [];

export function getMapElementApproach(
  _kind: WorkspaceElementKind,
  position: Point,
  facing: Direction,
): PointWithFacing {
  return {
    ...position,
    facing,
  };
}
