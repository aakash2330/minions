export enum Direction {
  Up = "up",
  Left = "left",
  Down = "down",
  Right = "right",
}

export const CHARACTER_TEXTURE_KEY = "player";
export const PLAYER_NAME = "player";
export const MINION_NPC_NAME = "minion";

export const CHARACTER_SPRITESHEET_PATH =
  "/assets/sprites/lpc-minionlike-spritesheet.png";
export const CHARACTER_FRAME_SIZE = 64;
export const CHARACTER_DISPLAY_SIZE = 64;

export const WALK_ANIMATION_ROWS: Record<Direction, number> = {
  [Direction.Up]: 8,
  [Direction.Left]: 9,
  [Direction.Down]: 10,
  [Direction.Right]: 11,
};

export function getIdleFrame(direction: Direction) {
  return WALK_ANIMATION_ROWS[direction] * 13;
}

export function getWalkAnimationKey(direction: Direction) {
  return `walk-${direction}`;
}

export function getDirectionFromVector(xAxis: number, yAxis: number) {
  if (Math.abs(xAxis) > Math.abs(yAxis)) {
    return xAxis < 0 ? Direction.Left : Direction.Right;
  }

  return yAxis < 0 ? Direction.Up : Direction.Down;
}
