import { Scene } from "phaser";

import { SessionController } from "./characters/SessionController";
import {
  PlayerController,
  type WorldBounds,
} from "./characters/PlayerController";
import { SessionInteractionType } from "./sessionInteractions";
import {
  CHARACTER_FRAME_SIZE,
  CHARACTER_SPRITESHEETS,
  CHARACTER_WALK_FRAMES_PER_DIRECTION,
  CHARACTER_VISIBLE_SIZE,
  Direction,
  getWalkAnimationKey,
  WALK_ANIMATION_ROWS,
} from "./characters/characterConfig";
import type { CanUseKeyboardInput } from "./input/keyboardControlGate";
import { getWorldMapCenter } from "@/features/world/map/coordinates";
import type { WorldMapConfig } from "@/features/world/map/types";
import type { PointWithFacing, SessionMapConfig } from "./sessionMapConfig";

const SESSION_INTERACTION_EVENT = "session.interaction";

export type SessionInteractionDetail = {
  sessionId: string;
  interactionType: SessionInteractionType;
};

export function dispatchSessionInteraction(detail: SessionInteractionDetail) {
  window.dispatchEvent(
    new CustomEvent<SessionInteractionDetail>(SESSION_INTERACTION_EVENT, {
      detail,
    }),
  );
}

export type WorldSceneOptions = {
  canUseKeyboardInput?: CanUseKeyboardInput;
  mapConfig: WorldMapConfig;
  sessions: SessionMapConfig[];
};

export class WorldScene extends Scene {
  private playerController?: PlayerController;
  private sessionControllers: SessionController[] = [];
  private sessionControllersById = new Map<string, SessionController>();

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
  }

  create() {
    this.cameras.main.setScroll(0, 0);
    this.createWalkAnimations();

    this.playerController = new PlayerController(
      this,
      getWorldBounds(this.options.mapConfig),
      this.getPlayerSpawn(),
      this.options.canUseKeyboardInput,
    );
    this.playerController.create();

    this.sessionControllers = this.options.sessions.map((config) => {
      const controller = new SessionController(this, {
        config,
      });

      controller.create();
      this.sessionControllersById.set(config.sessionId, controller);

      return controller;
    });
    window.addEventListener(
      SESSION_INTERACTION_EVENT,
      this.handleSessionInteraction,
    );

    this.events.once("shutdown", () => {
      window.removeEventListener(
        SESSION_INTERACTION_EVENT,
        this.handleSessionInteraction,
      );
      this.playerController?.destroy();
      this.sessionControllers.forEach((controller) => {
        controller.destroy();
      });
      this.sessionControllers = [];
      this.sessionControllersById.clear();
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
            start: row * CHARACTER_WALK_FRAMES_PER_DIRECTION,
            end: row * CHARACTER_WALK_FRAMES_PER_DIRECTION + 3,
          }),
          frameRate: 10,
          repeat: -1,
        });
      });
    });
  }

  private getPlayerSpawn(): PointWithFacing {
    const center = getWorldMapCenter(this.options.mapConfig);

    return {
      x: center.x - CHARACTER_VISIBLE_SIZE.width / 2,
      y: center.y - CHARACTER_VISIBLE_SIZE.height / 2,
      facing: Direction.Down,
    };
  }

  private handleSessionInteraction = (event: Event) => {
    const { sessionId, interactionType } = (
      event as CustomEvent<SessionInteractionDetail>
    ).detail;

    this.sessionControllersById
      .get(sessionId)
      ?.performInteraction(interactionType);
  };
}

function getWorldBounds(config: WorldMapConfig): WorldBounds {
  return {
    minX: 0,
    minY: 0,
    maxX: Math.max(0, config.canvas.width - CHARACTER_VISIBLE_SIZE.width),
    maxY: Math.max(0, config.canvas.height - CHARACTER_VISIBLE_SIZE.height),
  };
}
