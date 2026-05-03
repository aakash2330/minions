import { Scene } from "phaser";

import { MinionController } from "./characters/MinionController";
import { PlayerController } from "./characters/PlayerController";
import {
  CHARACTER_FRAME_SIZE,
  CHARACTER_SPRITESHEETS,
  Direction,
  getWalkAnimationKey,
  WALK_ANIMATION_ROWS,
} from "./characters/characterConfig";
import type { CanUseKeyboardInput } from "./input/keyboardControlGate";
import {
  MINION_MAP_CONFIGS,
  type MinionElementConfig,
  type MinionMapConfig,
} from "./minionMapConfig";

const DESK_ESSENTIALS_TEXTURE_KEY = "desk-essentials";
const DESK_ESSENTIALS_SPRITESHEET_PATH =
  "/assets/Pixel%20Life%20-%20Desk%20Essentials/spritesheet.png";
const WORK_DESK_SCALE = 1.75;

const DESK_ESSENTIAL_FRAMES = {
  desk: { x: 0, y: 1, width: 32, height: 30 },
  computer: { x: 103, y: 40, width: 20, height: 23 },
  keyboard: { x: 40, y: 43, width: 16, height: 9 },
  mouse: { x: 77, y: 44, width: 6, height: 6 },
  pencilCup: { x: 12, y: 42, width: 10, height: 12 },
} as const;

type DeskEssentialFrameName = keyof typeof DESK_ESSENTIAL_FRAMES;

export type WorldSceneOptions = {
  canUseKeyboardInput?: CanUseKeyboardInput;
  onMinionChat?: (config: MinionMapConfig) => void;
};

export class WorldScene extends Scene {
  private playerController?: PlayerController;
  private minionControllers: MinionController[] = [];

  constructor(private readonly options: WorldSceneOptions = {}) {
    super("world");
  }

  preload() {
    CHARACTER_SPRITESHEETS.forEach((spritesheet) => {
      this.load.spritesheet(spritesheet.key, spritesheet.path, {
        frameWidth: CHARACTER_FRAME_SIZE,
        frameHeight: CHARACTER_FRAME_SIZE,
      });
    });
    this.load.image(
      DESK_ESSENTIALS_TEXTURE_KEY,
      DESK_ESSENTIALS_SPRITESHEET_PATH,
    );
  }

  create() {
    this.createWalkAnimations();
    this.createDeskEssentialFrames();
    this.createConfiguredElements();

    this.playerController = new PlayerController(
      this,
      this.options.canUseKeyboardInput,
    );
    this.playerController.create();

    this.minionControllers = MINION_MAP_CONFIGS.map((config) => {
      const controller = new MinionController(this, {
        config,
        onChat: this.options.onMinionChat,
      });

      controller.create();

      return controller;
    });

    this.events.once("shutdown", () => {
      this.playerController?.destroy();
      this.minionControllers.forEach((controller) => {
        controller.destroy();
      });
      this.minionControllers = [];
    });
  }

  update(_: number, delta: number) {
    this.playerController?.update(delta);
  }

  private createWalkAnimations() {
    CHARACTER_SPRITESHEETS.forEach((spritesheet) => {
      Object.values(Direction).forEach((direction) => {
        const row = WALK_ANIMATION_ROWS[direction];
        const animationKey = getWalkAnimationKey(spritesheet.key, direction);

        if (this.anims.exists(animationKey)) {
          return;
        }

        this.anims.create({
          key: animationKey,
          frames: this.anims.generateFrameNumbers(spritesheet.key, {
            start: row * 13,
            end: row * 13 + 8,
          }),
          frameRate: 10,
          repeat: -1,
        });
      });
    });
  }

  private createDeskEssentialFrames() {
    const texture = this.textures.get(DESK_ESSENTIALS_TEXTURE_KEY);

    Object.entries(DESK_ESSENTIAL_FRAMES).forEach(([name, frame]) => {
      if (texture.has(name)) {
        return;
      }

      texture.add(name, 0, frame.x, frame.y, frame.width, frame.height);
    });
  }

  private createConfiguredElements() {
    MINION_MAP_CONFIGS.forEach((config) => {
      const workdesk = config.elements.workdesk;

      if (workdesk) {
        this.createWorkDesk(workdesk);
      }
    });
  }

  private createWorkDesk(workdesk: MinionElementConfig) {
    const { x, y } = workdesk.position;

    this.addDeskSprite("desk", x, y).setName(workdesk.id);
    this.addDeskSprite(
      "computer",
      x + 10 * WORK_DESK_SCALE,
      y - 18 * WORK_DESK_SCALE,
    ).setName(`${workdesk.id}-computer`);
    this.addDeskSprite(
      "keyboard",
      x + 4 * WORK_DESK_SCALE,
      y + 7 * WORK_DESK_SCALE,
    ).setName(`${workdesk.id}-keyboard`);
    this.addDeskSprite(
      "mouse",
      x + 23 * WORK_DESK_SCALE,
      y + 8 * WORK_DESK_SCALE,
    ).setName(`${workdesk.id}-mouse`);
    this.addDeskSprite(
      "pencilCup",
      x + 21 * WORK_DESK_SCALE,
      y + WORK_DESK_SCALE,
    ).setName(`${workdesk.id}-pencil-cup`);
  }

  private addDeskSprite(
    frameName: DeskEssentialFrameName,
    x: number,
    y: number,
  ) {
    return this.add
      .image(x, y, DESK_ESSENTIALS_TEXTURE_KEY, frameName)
      .setOrigin(0, 0)
      .setScale(WORK_DESK_SCALE);
  }
}
