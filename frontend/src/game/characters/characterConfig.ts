export enum Direction {
  Up = "up",
  UpLeft = "up-left",
  UpRight = "up-right",
  Left = "left",
  Down = "down",
  DownLeft = "down-left",
  DownRight = "down-right",
  Right = "right",
}

export enum CharacterSpriteName {
  Player = "player",
  Session = "session",
}

export const PLAYER_TEXTURE_KEY = "player";
export const SESSION_TEXTURE_KEY = "session";
export const PLAYER_NAME = CharacterSpriteName.Player;
export const SESSION_NPC_NAME = CharacterSpriteName.Session;

export const PLAYER_SPRITESHEET_PATH =
  "/assets/sprites/erisesra-character-walk.png";
export const SESSION_SPRITESHEET_PATH =
  "/assets/sprites/erisesra-character-walk.png";
export const CHARACTER_FRAME_SIZE = 32;
export const CHARACTER_DISPLAY_SIZE = 150;
export const CHARACTER_WALK_SPEED_PIXELS_PER_SECOND = 360;
export const CHARACTER_DEPTH_OFFSET = 10000;
export const CHARACTER_WALK_FRAMES_PER_DIRECTION = 4;
const CHARACTER_VISIBLE_FRAME_BOUNDS = {
  left: 8,
  top: 6,
  right: 24,
  bottom: 32,
} as const;
const CHARACTER_DISPLAY_SCALE = CHARACTER_DISPLAY_SIZE / CHARACTER_FRAME_SIZE;

// Character frames include transparent padding; movement coordinates use visible pixels.
export const CHARACTER_VISIBLE_BOUNDS = {
  left: CHARACTER_VISIBLE_FRAME_BOUNDS.left * CHARACTER_DISPLAY_SCALE,
  top: CHARACTER_VISIBLE_FRAME_BOUNDS.top * CHARACTER_DISPLAY_SCALE,
  right: CHARACTER_VISIBLE_FRAME_BOUNDS.right * CHARACTER_DISPLAY_SCALE,
  bottom: CHARACTER_VISIBLE_FRAME_BOUNDS.bottom * CHARACTER_DISPLAY_SCALE,
} as const;
export const CHARACTER_VISIBLE_SIZE = {
  width: CHARACTER_VISIBLE_BOUNDS.right - CHARACTER_VISIBLE_BOUNDS.left,
  height: CHARACTER_VISIBLE_BOUNDS.bottom - CHARACTER_VISIBLE_BOUNDS.top,
} as const;

export const CHARACTER_SPRITESHEETS = [
  {
    key: PLAYER_TEXTURE_KEY,
    path: PLAYER_SPRITESHEET_PATH,
  },
  {
    key: SESSION_TEXTURE_KEY,
    path: SESSION_SPRITESHEET_PATH,
  },
] as const;

export const WALK_ANIMATION_ROWS: Record<Direction, number> = {
  [Direction.Down]: 0,
  [Direction.DownLeft]: 1,
  [Direction.DownRight]: 1,
  [Direction.Right]: 2,
  [Direction.Left]: 2,
  [Direction.UpRight]: 3,
  [Direction.UpLeft]: 3,
  [Direction.Up]: 4,
};

export function getIdleFrame(direction: Direction) {
  return WALK_ANIMATION_ROWS[direction] * CHARACTER_WALK_FRAMES_PER_DIRECTION;
}

export function getWalkAnimationKey(textureKey: string, direction: Direction) {
  return `${textureKey}-walk-${direction}`;
}

export function getCharacterDepth(y: number) {
  return CHARACTER_DEPTH_OFFSET + y;
}

export function shouldFlipCharacterDirection(direction: Direction) {
  return [
    Direction.Left,
    Direction.DownLeft,
    Direction.UpLeft,
  ].includes(direction);
}

export function getDirectionFromVector(xAxis: number, yAxis: number) {
  const octant = Math.round(Math.atan2(yAxis, xAxis) / (Math.PI / 4));

  switch ((octant + 8) % 8) {
    case 0:
      return Direction.Right;
    case 1:
      return Direction.DownRight;
    case 2:
      return Direction.Down;
    case 3:
      return Direction.DownLeft;
    case 4:
      return Direction.Left;
    case 5:
      return Direction.UpLeft;
    case 6:
      return Direction.Up;
    default:
      return Direction.UpRight;
  }
}
