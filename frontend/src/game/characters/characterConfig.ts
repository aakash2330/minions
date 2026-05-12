export enum Direction {
  Up = "up",
  Left = "left",
  Down = "down",
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
  "/assets/sprites/lpc-player-spritesheet.png";
export const SESSION_SPRITESHEET_PATH =
  "/assets/sprites/lpc-session-spritesheet.png";
export const CHARACTER_FRAME_SIZE = 64;
export const CHARACTER_DISPLAY_SIZE = 64;
export const CHARACTER_WALK_SPEED_PIXELS_PER_SECOND = 240;

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
  [Direction.Up]: 8,
  [Direction.Left]: 9,
  [Direction.Down]: 10,
  [Direction.Right]: 11,
};

export function getIdleFrame(direction: Direction) {
  return WALK_ANIMATION_ROWS[direction] * 13;
}

export function getWalkAnimationKey(textureKey: string, direction: Direction) {
  return `${textureKey}-walk-${direction}`;
}

export function getDirectionFromVector(xAxis: number, yAxis: number) {
  if (Math.abs(xAxis) > Math.abs(yAxis)) {
    return xAxis < 0 ? Direction.Left : Direction.Right;
  }

  return yAxis < 0 ? Direction.Up : Direction.Down;
}
