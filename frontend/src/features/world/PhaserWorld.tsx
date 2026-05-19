import { type CSSProperties, useEffect, useRef } from "react";
import { AUTO, Game } from "phaser";

import { usePanelStore } from "@/features/panel/stores/panelStore";
import { canUseGameKeyboardInput } from "@/game/input/keyboardControlGate";
import type { SessionMapConfig } from "@/game/sessionMapConfig";
import { WorkspaceElementKind } from "@/game/workspaceElementKind";
import { WorldScene } from "@/game/WorldScene";
import { WORLD_MAP_ASSETS } from "./map/assets.generated";
import {
  WORLD_MAP_VIEW_SCALE,
  getWorldMapViewport,
} from "./map/coordinates";
import type { WorldMapConfig, WorldMapItem } from "./map/types";

const ASSET_BY_FILE_NAME = new Map(
  WORLD_MAP_ASSETS.map((asset) => [asset.fileName, asset]),
);

type PhaserWorldProps = {
  mapConfig: WorldMapConfig;
  sessions: SessionMapConfig[];
};

export function PhaserWorld({ mapConfig, sessions }: PhaserWorldProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const gameRef = useRef<Game | null>(null);
  const viewport = getWorldMapViewport(mapConfig);

  useEffect(() => {
    if (!parentRef.current) {
      return;
    }

    const game = new Game({
      type: AUTO,
      parent: parentRef.current,
      transparent: true,
      width: mapConfig.canvas.width,
      height: mapConfig.canvas.height,
      render: {
        pixelArt: true,
      },
      scene: [
        new WorldScene({
          canUseKeyboardInput: canUseWorldKeyboardInput,
          mapConfig,
          sessions,
        }),
      ],
    });
    gameRef.current = game;

    return () => {
      game.destroy(true);
      if (gameRef.current === game) {
        gameRef.current = null;
      }
    };
  }, [mapConfig, sessions]);

  return (
    <div
      className="world-game"
      style={{
        aspectRatio: `${viewport.width} / ${viewport.height}`,
        maxWidth: "100%",
        width: `${viewport.width}px`,
      }}
    >
      <div
        className="world-map-stage-size"
        style={{
          height: viewport.height,
          width: viewport.width,
        }}
      >
        <div
          className="world-map-stage"
          data-studio-scene={mapConfig.name === "AI Crew Studio"}
          style={
            {
              "--world-map-grid-size": `${mapConfig.canvas.gridSize}px`,
              backgroundColor: mapConfig.canvas.background,
              height: mapConfig.canvas.height,
              transform: `scale(${WORLD_MAP_VIEW_SCALE})`,
              width: mapConfig.canvas.width,
            } as CSSProperties
          }
        >
          {mapConfig.items.map((item) => {
            const asset = ASSET_BY_FILE_NAME.get(item.assetId);

            if (!asset) {
              return null;
            }

            return (
              <div
                className="world-map-item"
                key={item.id}
                style={{
                  height: item.height,
                  left: item.x,
                  top: item.y,
                  width: item.width,
                  zIndex: getWorldMapItemDepth(item),
                }}
              >
                <img alt="" draggable={false} src={asset.url} />
              </div>
            );
          })}
        </div>
      </div>
      <div className="phaser-game world-actors-layer" ref={parentRef} />
    </div>
  );
}

const WORLD_MAP_KIND_LAYER: Partial<Record<WorkspaceElementKind, number>> = {
  [WorkspaceElementKind.Rug]: 0,
  [WorkspaceElementKind.Sofa]: 10,
  [WorkspaceElementKind.Desk]: 20,
  [WorkspaceElementKind.Table]: 25,
  [WorkspaceElementKind.Stool]: 30,
  [WorkspaceElementKind.Chair]: 35,
  [WorkspaceElementKind.Computer]: 40,
  [WorkspaceElementKind.Monitor]: 40,
  [WorkspaceElementKind.Keyboard]: 45,
  [WorkspaceElementKind.Laptop]: 45,
  [WorkspaceElementKind.Lamp]: 50,
  [WorkspaceElementKind.BookStack]: 50,
  [WorkspaceElementKind.Mug]: 50,
  [WorkspaceElementKind.Plant]: 55,
  [WorkspaceElementKind.Cactus]: 55,
};

function getWorldMapItemDepth(item: WorldMapItem) {
  return (WORLD_MAP_KIND_LAYER[item.kind] ?? 100) * 1000 + item.y;
}

function canUseWorldKeyboardInput() {
  return canUseGameKeyboardInput({
    disabled: usePanelStore.getState().isOpen,
  });
}
