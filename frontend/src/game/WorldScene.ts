import { Scene } from "phaser";

import { SessionController } from "./characters/SessionController";
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
  MapElementKind,
  STATIC_MAP_ELEMENTS,
  WORK_DESK_SCALE,
  type SessionElementConfig,
  type SessionMapConfig,
  type StaticMapElementConfig,
} from "./sessionMapConfig";

const DESK_ESSENTIALS_TEXTURE_KEY = "desk-essentials";
const DESK_ESSENTIALS_SPRITESHEET_PATH =
  "/assets/Pixel%20Life%20-%20Desk%20Essentials/spritesheet.png";

enum DeskEssentialSpriteFrame {
  Desk = "desk",
  Computer = "computer",
  Keyboard = "keyboard",
  Mouse = "mouse",
  PencilCup = "pencilCup",
}

enum WorkDeskSpritePart {
  Desk = "desk",
  Computer = "computer",
  Keyboard = "keyboard",
  Mouse = "mouse",
  PencilCup = "pencil-cup",
}

const WORK_DESK_SPRITE_NAME_SUFFIXES: Partial<
  Record<WorkDeskSpritePart, string>
> = {
  [WorkDeskSpritePart.Computer]: WorkDeskSpritePart.Computer,
  [WorkDeskSpritePart.Keyboard]: WorkDeskSpritePart.Keyboard,
  [WorkDeskSpritePart.Mouse]: WorkDeskSpritePart.Mouse,
  [WorkDeskSpritePart.PencilCup]: WorkDeskSpritePart.PencilCup,
};

const DESK_ESSENTIAL_FRAMES: Record<
  DeskEssentialSpriteFrame,
  { x: number; y: number; width: number; height: number }
> = {
  [DeskEssentialSpriteFrame.Desk]: { x: 0, y: 1, width: 32, height: 30 },
  [DeskEssentialSpriteFrame.Computer]: { x: 103, y: 40, width: 20, height: 23 },
  [DeskEssentialSpriteFrame.Keyboard]: { x: 40, y: 43, width: 16, height: 9 },
  [DeskEssentialSpriteFrame.Mouse]: { x: 77, y: 44, width: 6, height: 6 },
  [DeskEssentialSpriteFrame.PencilCup]: {
    x: 12,
    y: 42,
    width: 10,
    height: 12,
  },
};

export type WorldSceneOptions = {
  canUseKeyboardInput?: CanUseKeyboardInput;
  sessions: SessionMapConfig[];
  staticElements?: StaticMapElementConfig[];
};

export class WorldScene extends Scene {
  private playerController?: PlayerController;
  private sessionControllers: SessionController[] = [];

  constructor(private readonly options: WorldSceneOptions) {
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
    this.createStaticElements();
    this.createSessionElements();

    this.playerController = new PlayerController(
      this,
      this.options.canUseKeyboardInput,
    );
    this.playerController.create();

    this.sessionControllers = this.options.sessions.map((config) => {
      const controller = new SessionController(this, {
        config,
      });

      controller.create();

      return controller;
    });

    this.events.once("shutdown", () => {
      this.playerController?.destroy();
      this.sessionControllers.forEach((controller) => {
        controller.destroy();
      });
      this.sessionControllers = [];
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

  private createStaticElements() {
    (this.options.staticElements ?? STATIC_MAP_ELEMENTS).forEach((element) => {
      this.createStaticElement(element);
    });
  }

  private createSessionElements() {
    this.options.sessions.forEach((config) => {
      const workdesk = config.elements[MapElementKind.Workdesk];

      if (workdesk) {
        this.createSessionWorkDesk(workdesk);
      }
    });
  }

  private createStaticElement(element: StaticMapElementConfig) {
    switch (element.kind) {
      case MapElementKind.Workdesk:
        this.createStaticWorkDesk(element);
        return;
    }
  }

  private createStaticWorkDesk(workdesk: StaticMapElementConfig) {
    this.createWorkDeskSprites(workdesk).forEach((sprite) => {
      const suffix = WORK_DESK_SPRITE_NAME_SUFFIXES[sprite.part];

      sprite.gameObject.setName(
        suffix ? `${workdesk.id}-${suffix}` : workdesk.id,
      );
    });
  }

  private createSessionWorkDesk(workdesk: SessionElementConfig) {
    this.createWorkDeskSprites(workdesk).forEach((sprite) => {
      sprite.gameObject.setName(
        this.getWorkDeskSpriteName(workdesk.id, sprite.part),
      );
    });
  }

  private createWorkDeskSprites(
    workdesk: SessionElementConfig | StaticMapElementConfig,
  ) {
    const { x, y } = workdesk.position;

    return [
      {
        gameObject: this.addDeskSprite(DeskEssentialSpriteFrame.Desk, x, y),
        part: WorkDeskSpritePart.Desk,
      },
      {
        gameObject: this.addDeskSprite(
          DeskEssentialSpriteFrame.Computer,
          x + 10 * WORK_DESK_SCALE,
          y - 18 * WORK_DESK_SCALE,
        ),
        part: WorkDeskSpritePart.Computer,
      },
      {
        gameObject: this.addDeskSprite(
          DeskEssentialSpriteFrame.Keyboard,
          x + 4 * WORK_DESK_SCALE,
          y + 7 * WORK_DESK_SCALE,
        ),
        part: WorkDeskSpritePart.Keyboard,
      },
      {
        gameObject: this.addDeskSprite(
          DeskEssentialSpriteFrame.Mouse,
          x + 23 * WORK_DESK_SCALE,
          y + 8 * WORK_DESK_SCALE,
        ),
        part: WorkDeskSpritePart.Mouse,
      },
      {
        gameObject: this.addDeskSprite(
          DeskEssentialSpriteFrame.PencilCup,
          x + 21 * WORK_DESK_SCALE,
          y + WORK_DESK_SCALE,
        ),
        part: WorkDeskSpritePart.PencilCup,
      },
    ];
  }

  private getWorkDeskSpriteName(
    workdeskId: string,
    spritePart: WorkDeskSpritePart,
  ) {
    const suffix = WORK_DESK_SPRITE_NAME_SUFFIXES[spritePart];

    return suffix ? `${workdeskId}-${suffix}` : workdeskId;
  }

  private addDeskSprite(
    frameName: DeskEssentialSpriteFrame,
    x: number,
    y: number,
  ) {
    return this.add
      .image(x, y, DESK_ESSENTIALS_TEXTURE_KEY, frameName)
      .setOrigin(0, 0)
      .setScale(WORK_DESK_SCALE);
  }
}
