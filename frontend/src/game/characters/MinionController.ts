import { Math as PhaserMath, Scene } from "phaser";

import { usePanelStore } from "@/features/panel/stores/panelStore";
import {
  CHARACTER_DISPLAY_SIZE,
  CHARACTER_WALK_SPEED_PIXELS_PER_SECOND,
  Direction,
  getDirectionFromVector,
  getIdleFrame,
  getWalkAnimationKey,
  MINION_TEXTURE_KEY,
} from "./characterConfig";
import {
  INTERACTIVE_MAP_ELEMENT_KINDS,
  type MapElementKind,
  type MinionMapConfig,
} from "../minionMapConfig";

const MINION_HIGHLIGHT_TINT = 0xffdf6e;
const ACTION_DIALOG_WIDTH = 152;
const ACTION_DIALOG_MARGIN = 16;
const ACTION_BUTTON_HEIGHT = 28;
const ACTION_BUTTON_GAP = 6;
const ACTION_BUTTON_X = 8;
const ACTION_BUTTON_Y = 10;
const ACTION_DIALOG_VERTICAL_PADDING = 10;
const MINION_ACTION_DIALOG_OPEN_EVENT = "minion-action-dialog-open";

type MinionAction = {
  id: string;
  label: string;
  onSelect: () => void;
};

type MinionControllerOptions = {
  config: MinionMapConfig;
};

export class MinionController {
  private sprite?: Phaser.GameObjects.Sprite;
  private actionDialog?: Phaser.GameObjects.Container;
  private movementTween?: Phaser.Tweens.Tween;
  private currentDirection = Direction.Down;

  constructor(
    private readonly scene: Scene,
    private readonly options: MinionControllerOptions,
  ) {}

  create() {
    const { config } = this.options;

    this.currentDirection = config.current.facing;
    this.sprite = this.scene.add
      .sprite(
        config.current.x,
        config.current.y,
        MINION_TEXTURE_KEY,
        getIdleFrame(this.currentDirection),
      )
      .setName(config.minionId)
      .setDisplaySize(CHARACTER_DISPLAY_SIZE, CHARACTER_DISPLAY_SIZE)
      .setInteractive({
        pixelPerfect: true,
        alphaTolerance: 1,
        useHandCursor: true,
      });

    this.sprite.on("pointerover", this.handlePointerOver, this);
    this.sprite.on("pointerout", this.handlePointerOut, this);
    this.sprite.on("pointerdown", this.handlePointerDown, this);
    this.scene.input.on("pointerdown", this.hideActionDialog, this);
    this.scene.events.on(
      MINION_ACTION_DIALOG_OPEN_EVENT,
      this.handleActionDialogOpened,
      this,
    );
  }

  destroy() {
    this.scene.input.off("pointerdown", this.hideActionDialog, this);
    this.scene.events.off(
      MINION_ACTION_DIALOG_OPEN_EVENT,
      this.handleActionDialogOpened,
      this,
    );
    this.movementTween?.stop();
    this.actionDialog?.destroy();
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
    this.toggleActionDialog();
  }

  private toggleActionDialog() {
    if (this.actionDialog) {
      this.hideActionDialog();
      return;
    }

    this.showActionDialog();
  }

  private showActionDialog() {
    const actions = this.getActions();

    if (!this.sprite) {
      return;
    }

    this.scene.events.emit(
      MINION_ACTION_DIALOG_OPEN_EVENT,
      this.options.config.minionId,
    );

    const dialogHeight = this.getActionDialogHeight(actions.length);
    const dialogX = PhaserMath.Clamp(
      this.sprite.x - ACTION_DIALOG_WIDTH / 2,
      ACTION_DIALOG_MARGIN,
      this.scene.scale.width - ACTION_DIALOG_WIDTH - ACTION_DIALOG_MARGIN,
    );
    const dialogY = PhaserMath.Clamp(
      this.sprite.y - CHARACTER_DISPLAY_SIZE / 2 - dialogHeight - 14,
      ACTION_DIALOG_MARGIN,
      this.scene.scale.height - dialogHeight - ACTION_DIALOG_MARGIN,
    );
    const pointerX = PhaserMath.Clamp(
      this.sprite.x - dialogX,
      18,
      ACTION_DIALOG_WIDTH - 18,
    );
    const background = this.scene.add.graphics();

    background.fillStyle(0x111719, 0.96);
    background.lineStyle(1, 0xffdf6e, 0.9);
    background.fillRoundedRect(0, 0, ACTION_DIALOG_WIDTH, dialogHeight, 8);
    background.strokeRoundedRect(0, 0, ACTION_DIALOG_WIDTH, dialogHeight, 8);
    background.fillStyle(0x111719, 0.96);
    background.fillTriangle(
      pointerX - 7,
      dialogHeight - 1,
      pointerX + 7,
      dialogHeight - 1,
      pointerX,
      dialogHeight + 8,
    );
    background.lineStyle(1, 0xffdf6e, 0.9);
    background.lineBetween(
      pointerX - 7,
      dialogHeight,
      pointerX,
      dialogHeight + 8,
    );
    background.lineBetween(
      pointerX,
      dialogHeight + 8,
      pointerX + 7,
      dialogHeight,
    );

    this.actionDialog = this.scene.add
      .container(dialogX, dialogY, [
        background,
        ...actions.map((action, index) =>
          this.createActionButton(
            action.label,
            ACTION_BUTTON_Y + index * (ACTION_BUTTON_HEIGHT + ACTION_BUTTON_GAP),
            () => {
              this.hideActionDialog();
              action.onSelect();
            },
          ),
        ),
      ])
      .setDepth(1000);
  }

  private createActionButton(label: string, y: number, onSelect: () => void) {
    const container = this.scene.add.container(ACTION_BUTTON_X, y);
    const width = ACTION_DIALOG_WIDTH - ACTION_BUTTON_X * 2;
    const background = this.scene.add
      .rectangle(0, 0, width, ACTION_BUTTON_HEIGHT, 0x243037, 0.92)
      .setOrigin(0, 0)
      .setInteractive({ useHandCursor: true });
    const text = this.scene.add
      .text(12, 6, label, {
        color: "#f8f6df",
        fontFamily: "Geist Variable, sans-serif",
        fontSize: "13px",
      })
      .setOrigin(0, 0);

    background.on("pointerover", () => {
      background.setFillStyle(0x3a4539, 1);
      text.setColor("#fff7b8");
    });
    background.on("pointerout", () => {
      background.setFillStyle(0x243037, 0.92);
      text.setColor("#f8f6df");
    });
    background.on(
      "pointerdown",
      (
        _pointer: Phaser.Input.Pointer,
        _localX: number,
        _localY: number,
        event: Phaser.Types.Input.EventData,
      ) => {
        event.stopPropagation();
        onSelect();
      },
    );

    return container.add([background, text]);
  }

  private getActions(): MinionAction[] {
    const elementActions = INTERACTIVE_MAP_ELEMENT_KINDS.flatMap(
      (elementKind) => {
        const element = this.options.config.elements[elementKind];

        if (!element) {
          return [];
        }

        return [
          {
            id: `go-to-${elementKind}`,
            label: `Go to ${element.label}`,
            onSelect: () => {
              this.goToElement(elementKind);
            },
          },
        ];
      },
    );

    return [
      {
        id: "chat",
        label: "Chat",
        onSelect: () => {
          usePanelStore.getState().open({
            type: "minion-chat",
            minionId: this.options.config.minionId,
          });
        },
      },
      ...elementActions,
    ];
  }

  private getActionDialogHeight(actionCount: number) {
    return (
      ACTION_DIALOG_VERTICAL_PADDING * 2 +
      actionCount * ACTION_BUTTON_HEIGHT +
      Math.max(0, actionCount - 1) * ACTION_BUTTON_GAP
    );
  }

  private handleActionDialogOpened(minionId: string) {
    if (minionId !== this.options.config.minionId) {
      this.hideActionDialog();
    }
  }

  private hideActionDialog() {
    this.actionDialog?.destroy();
    this.actionDialog = undefined;
  }

  goToElement(elementKind: MapElementKind) {
    const element = this.options.config.elements[elementKind];

    if (!element) {
      return;
    }

    this.moveTo(element.approach);
  }

  private moveTo(target: { x: number; y: number; facing: Direction }) {
    if (!this.sprite) {
      return;
    }

    this.movementTween?.stop();

    const xAxis = target.x - this.sprite.x;
    const yAxis = target.y - this.sprite.y;
    const distance = Math.hypot(xAxis, yAxis);

    if (distance < 4) {
      this.stopWalking(target.facing);
      return;
    }

    this.playWalk(getDirectionFromVector(xAxis, yAxis));

    this.movementTween = this.scene.tweens.add({
      targets: this.sprite,
      x: target.x,
      y: target.y,
      duration: (distance / CHARACTER_WALK_SPEED_PIXELS_PER_SECOND) * 1000,
      ease: "Linear",
      onComplete: () => {
        this.movementTween = undefined;
        this.stopWalking(target.facing);
      },
    });
  }

  private playWalk(direction: Direction) {
    if (!this.sprite) {
      return;
    }

    this.currentDirection = direction;
    this.sprite.play(getWalkAnimationKey(MINION_TEXTURE_KEY, direction), true);
  }

  private stopWalking(direction = this.currentDirection) {
    this.currentDirection = direction;
    this.sprite?.anims.stop();
    this.sprite?.setFrame(getIdleFrame(direction));
  }
}
