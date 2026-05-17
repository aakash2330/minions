import {
  CHARACTER_DISPLAY_SIZE,
  Direction,
} from "./characters/characterConfig";

export enum MapElementKind {
  PersonalTable = "personal-table",
  MeetingTable = "meeting-table",
  Workdesk = "workdesk",
}

export const INTERACTIVE_MAP_ELEMENT_KINDS = [
  MapElementKind.PersonalTable,
  MapElementKind.MeetingTable,
] as const;

export type Point = {
  x: number;
  y: number;
};

export type PointWithFacing = Point & {
  facing: Direction;
};

export type SessionElementConfig = {
  id: string;
  kind: MapElementKind;
  label: string;
  position: Point;
  approach: PointWithFacing;
};

export type StaticMapElementConfig = {
  id: string;
  kind: MapElementKind;
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
  elements: Partial<Record<MapElementKind, SessionElementConfig>>;
};

export const STATIC_MAP_ELEMENTS: StaticMapElementConfig[] = [];

export const WORK_DESK_SCALE = 1.75;

const WORK_DESK_FRAME_SIZE = {
  width: 32,
  height: 30,
};
const WORK_DESK_SESSION_FRONT_OVERLAP = 40;

export function isMapElementKind(kind: string): kind is MapElementKind {
  return (INTERACTIVE_MAP_ELEMENT_KINDS as readonly string[]).includes(kind);
}

export function getMapElementApproach(
  kind: MapElementKind,
  position: Point,
  facing: Direction,
): PointWithFacing {
  switch (kind) {
    case MapElementKind.PersonalTable:
    case MapElementKind.MeetingTable:
      return {
        ...position,
        facing,
      };
    case MapElementKind.Workdesk:
      return {
        x: position.x + (WORK_DESK_FRAME_SIZE.width * WORK_DESK_SCALE) / 2,
        y:
          position.y +
          WORK_DESK_FRAME_SIZE.height * WORK_DESK_SCALE +
          CHARACTER_DISPLAY_SIZE / 2 -
          WORK_DESK_SESSION_FRONT_OVERLAP,
        facing,
      };
  }
}
