import { Scene } from "phaser";

import { MinionController } from "./characters/MinionController";
import { PlayerController } from "./characters/PlayerController";
import {
  CHARACTER_FRAME_SIZE,
  CHARACTER_SPRITESHEET_PATH,
  CHARACTER_TEXTURE_KEY,
  Direction,
  getWalkAnimationKey,
  WALK_ANIMATION_ROWS,
} from "./characters/characterConfig";
import type { CanUseKeyboardInput } from "./input/keyboardControlGate";

export type WorldSceneOptions = {
  canUseKeyboardInput?: CanUseKeyboardInput;
  onMinionInteract?: () => void;
};

export class WorldScene extends Scene {
  private playerController?: PlayerController;
  private minionController?: MinionController;

  constructor(private readonly options: WorldSceneOptions = {}) {
    super("world");
  }

  preload() {
    this.load.spritesheet(CHARACTER_TEXTURE_KEY, CHARACTER_SPRITESHEET_PATH, {
      frameWidth: CHARACTER_FRAME_SIZE,
      frameHeight: CHARACTER_FRAME_SIZE,
    });
  }

  create() {
    this.createWalkAnimations();

    this.playerController = new PlayerController(
      this,
      this.options.canUseKeyboardInput,
    );
    this.playerController.create();

    this.minionController = new MinionController(
      this,
      this.options.onMinionInteract,
    );
    this.minionController.create();

    this.events.once("shutdown", () => {
      this.playerController?.destroy();
      this.minionController?.destroy();
    });
  }

  update(_: number, delta: number) {
    this.playerController?.update(delta);
  }

  private createWalkAnimations() {
    Object.values(Direction).forEach((direction) => {
      const row = WALK_ANIMATION_ROWS[direction];
      const animationKey = getWalkAnimationKey(direction);

      if (this.anims.exists(animationKey)) {
        return;
      }

      this.anims.create({
        key: animationKey,
        frames: this.anims.generateFrameNumbers(CHARACTER_TEXTURE_KEY, {
          start: row * 13,
          end: row * 13 + 8,
        }),
        frameRate: 10,
        repeat: -1,
      });
    });
  }
}
