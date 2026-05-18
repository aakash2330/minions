import { Input, Math as PhaserMath, Scene } from "phaser";

import {
  CHARACTER_DISPLAY_SIZE,
  CHARACTER_VISIBLE_BOUNDS,
  CHARACTER_VISIBLE_SIZE,
  CHARACTER_WALK_SPEED_PIXELS_PER_SECOND,
  Direction,
  getCharacterDepth,
  getDirectionFromVector,
  getIdleFrame,
  getWalkAnimationKey,
  PLAYER_NAME,
  PLAYER_TEXTURE_KEY,
  shouldFlipCharacterDirection,
} from "./characterConfig";
import type { CanUseKeyboardInput } from "../input/keyboardControlGate";
import type { PointWithFacing } from "../sessionMapConfig";

const MOVEMENT_CAPTURE_KEY_CODES = [
  Input.Keyboard.KeyCodes.UP,
  Input.Keyboard.KeyCodes.DOWN,
  Input.Keyboard.KeyCodes.LEFT,
  Input.Keyboard.KeyCodes.RIGHT,
  Input.Keyboard.KeyCodes.SPACE,
  Input.Keyboard.KeyCodes.SHIFT,
  Input.Keyboard.KeyCodes.W,
  Input.Keyboard.KeyCodes.A,
  Input.Keyboard.KeyCodes.S,
  Input.Keyboard.KeyCodes.D,
];

type WasdKeys = Record<"W" | "A" | "S" | "D", Phaser.Input.Keyboard.Key>;

type MovementKeys = {
  cursors: Phaser.Types.Input.Keyboard.CursorKeys;
  wasd: WasdKeys;
};

export type WorldBounds = {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
};

const PLAYER_TINT = 0x4f8cff;

export class PlayerController {
  private sprite?: Phaser.GameObjects.Sprite;
  private movementKeys?: MovementKeys;
  private currentDirection = Direction.Down;
  private keyboardMoving = false;

  constructor(
    private readonly scene: Scene,
    private readonly worldBounds: WorldBounds,
    private readonly spawn: PointWithFacing,
    private readonly canUseKeyboardInput: CanUseKeyboardInput = () => true,
  ) {}

  create() {
    this.currentDirection = this.spawn.facing;
    this.sprite = this.scene.add
      .sprite(
        this.getSpriteXFromVisibleX(this.spawn.x),
        this.getSpriteYFromVisibleY(this.spawn.y),
        PLAYER_TEXTURE_KEY,
        getIdleFrame(this.currentDirection),
      )
      .setName(PLAYER_NAME)
      .setOrigin(0, 0)
      .setFlipX(shouldFlipCharacterDirection(this.currentDirection))
      .setTint(PLAYER_TINT)
      .setDisplaySize(CHARACTER_DISPLAY_SIZE, CHARACTER_DISPLAY_SIZE)
      .setDepth(getCharacterDepth(this.getVisibleDepthY()));

    const keyboard = this.scene.input.keyboard;

    if (keyboard) {
      keyboard.removeCapture(MOVEMENT_CAPTURE_KEY_CODES);

      this.movementKeys = {
        cursors: keyboard.addKeys(
          {
            up: Input.Keyboard.KeyCodes.UP,
            down: Input.Keyboard.KeyCodes.DOWN,
            left: Input.Keyboard.KeyCodes.LEFT,
            right: Input.Keyboard.KeyCodes.RIGHT,
            space: Input.Keyboard.KeyCodes.SPACE,
            shift: Input.Keyboard.KeyCodes.SHIFT,
          },
          false,
        ) as Phaser.Types.Input.Keyboard.CursorKeys,
        wasd: keyboard.addKeys("W,A,S,D", false) as WasdKeys,
      };
    }
  }

  update(delta: number) {
    if (!this.sprite || !this.movementKeys) {
      return;
    }

    if (!this.canUseKeyboardInput()) {
      if (this.keyboardMoving) {
        this.stopWalking();
      }

      this.keyboardMoving = false;
      return;
    }

    const { cursors, wasd } = this.movementKeys;
    const xAxis =
      Number(wasd.D.isDown || cursors.right.isDown) -
      Number(wasd.A.isDown || cursors.left.isDown);
    const yAxis =
      Number(wasd.S.isDown || cursors.down.isDown) -
      Number(wasd.W.isDown || cursors.up.isDown);

    if (xAxis === 0 && yAxis === 0) {
      if (this.keyboardMoving) {
        this.stopWalking();
      }

      this.keyboardMoving = false;
      return;
    }

    this.keyboardMoving = true;

    const direction = getDirectionFromVector(xAxis, yAxis);
    this.playWalk(direction);

    const distance = CHARACTER_WALK_SPEED_PIXELS_PER_SECOND * (delta / 1000);
    const length = Math.hypot(xAxis, yAxis);
    const position = this.getVisiblePosition();
    const nextX = position.x + (xAxis / length) * distance;
    const nextY = position.y + (yAxis / length) * distance;

    const x = PhaserMath.Clamp(
      nextX,
      this.worldBounds.minX,
      this.worldBounds.maxX,
    );
    const y = PhaserMath.Clamp(
      nextY,
      this.worldBounds.minY,
      this.worldBounds.maxY,
    );

    this.setVisiblePosition(x, y);
  }

  destroy() {
    this.sprite?.destroy();
  }

  private playWalk(direction: Direction) {
    if (!this.sprite) {
      return;
    }

    this.currentDirection = direction;
    this.sprite
      .setFlipX(shouldFlipCharacterDirection(direction))
      .play(getWalkAnimationKey(PLAYER_TEXTURE_KEY, direction), true);
  }

  private stopWalking() {
    this.sprite?.anims.stop();
    this.sprite
      ?.setFlipX(shouldFlipCharacterDirection(this.currentDirection))
      .setFrame(getIdleFrame(this.currentDirection));
  }

  private getVisiblePosition() {
    return {
      x: (this.sprite?.x ?? 0) + CHARACTER_VISIBLE_BOUNDS.left,
      y: (this.sprite?.y ?? 0) + CHARACTER_VISIBLE_BOUNDS.top,
    };
  }

  private setVisiblePosition(x: number, y: number) {
    this.sprite
      ?.setPosition(
        this.getSpriteXFromVisibleX(x),
        this.getSpriteYFromVisibleY(y),
      )
      .setDepth(getCharacterDepth(y + CHARACTER_VISIBLE_SIZE.height));
  }

  private getSpriteXFromVisibleX(x: number) {
    return x - CHARACTER_VISIBLE_BOUNDS.left;
  }

  private getSpriteYFromVisibleY(y: number) {
    return y - CHARACTER_VISIBLE_BOUNDS.top;
  }

  private getVisibleDepthY() {
    return this.getVisiblePosition().y + CHARACTER_VISIBLE_SIZE.height;
  }
}
