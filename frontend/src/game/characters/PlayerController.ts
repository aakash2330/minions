import { Math as PhaserMath, Scene } from "phaser";

import {
  CHARACTER_DISPLAY_SIZE,
  CHARACTER_TEXTURE_KEY,
  Direction,
  getDirectionFromVector,
  getIdleFrame,
  getWalkAnimationKey,
  PLAYER_NAME,
} from "./characterConfig";
import type { CanUseKeyboardInput } from "../input/keyboardControlGate";

const PLAYER_WALK_SPEED_PIXELS_PER_SECOND = 240;

type WasdKeys = Record<"W" | "A" | "S" | "D", Phaser.Input.Keyboard.Key>;

type MovementKeys = {
  cursors: Phaser.Types.Input.Keyboard.CursorKeys;
  wasd: WasdKeys;
};

export class PlayerController {
  private sprite?: Phaser.GameObjects.Sprite;
  private movementKeys?: MovementKeys;
  private currentDirection = Direction.Down;
  private keyboardMoving = false;

  constructor(
    private readonly scene: Scene,
    private readonly canUseKeyboardInput: CanUseKeyboardInput = () => true,
  ) {}

  create() {
    this.sprite = this.scene.add
      .sprite(480, 270, CHARACTER_TEXTURE_KEY, getIdleFrame(Direction.Down))
      .setName(PLAYER_NAME)
      .setDisplaySize(CHARACTER_DISPLAY_SIZE, CHARACTER_DISPLAY_SIZE);

    const keyboard = this.scene.input.keyboard;

    if (keyboard) {
      this.movementKeys = {
        cursors: keyboard.createCursorKeys(),
        wasd: keyboard.addKeys("W,A,S,D") as WasdKeys,
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

    const distance = PLAYER_WALK_SPEED_PIXELS_PER_SECOND * (delta / 1000);
    const length = Math.hypot(xAxis, yAxis);
    const nextX = this.sprite.x + (xAxis / length) * distance;
    const nextY = this.sprite.y + (yAxis / length) * distance;

    this.sprite.setPosition(
      PhaserMath.Clamp(nextX, 32, 928),
      PhaserMath.Clamp(nextY, 48, 508),
    );
  }

  destroy() {
    this.sprite?.destroy();
  }

  private playWalk(direction: Direction) {
    if (!this.sprite) {
      return;
    }

    this.currentDirection = direction;
    this.sprite.play(getWalkAnimationKey(direction), true);
  }

  private stopWalking() {
    this.sprite?.anims.stop();
    this.sprite?.setFrame(getIdleFrame(this.currentDirection));
  }
}
