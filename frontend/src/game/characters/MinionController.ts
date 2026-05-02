import { Scene } from "phaser";

import {
  CHARACTER_DISPLAY_SIZE,
  CHARACTER_TEXTURE_KEY,
  Direction,
  getIdleFrame,
  MINION_NPC_NAME,
} from "./characterConfig";

const MINION_HIGHLIGHT_TINT = 0xffdf6e;

export class MinionController {
  private sprite?: Phaser.GameObjects.Sprite;

  constructor(
    private readonly scene: Scene,
    private readonly onInteract?: () => void,
  ) {}

  create() {
    this.sprite = this.scene.add
      .sprite(580, 300, CHARACTER_TEXTURE_KEY, getIdleFrame(Direction.Down))
      .setName(MINION_NPC_NAME)
      .setDisplaySize(CHARACTER_DISPLAY_SIZE, CHARACTER_DISPLAY_SIZE)
      .setInteractive({
        pixelPerfect: true,
        alphaTolerance: 1,
        useHandCursor: true,
      });

    this.sprite.on("pointerover", this.handlePointerOver, this);
    this.sprite.on("pointerout", this.handlePointerOut, this);
    this.sprite.on("pointerdown", this.handlePointerDown, this);
  }

  destroy() {
    this.sprite?.destroy();
  }

  private handlePointerOver() {
    this.sprite?.setTint(MINION_HIGHLIGHT_TINT);
  }

  private handlePointerOut() {
    this.sprite?.clearTint();
  }

  private handlePointerDown(
    _pointer: Phaser.Input.Pointer,
    _localX: number,
    _localY: number,
    event: Phaser.Types.Input.EventData,
  ) {
    event.stopPropagation();
    this.onInteract?.();
  }
}
