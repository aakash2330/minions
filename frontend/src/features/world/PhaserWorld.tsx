import { type CSSProperties, useEffect, useRef } from "react";
import { AUTO, Game } from "phaser";

import { usePanelStore } from "@/features/panel/stores/panelStore";
import { canUseGameKeyboardInput } from "@/game/input/keyboardControlGate";
import type { SessionMapConfig } from "@/game/sessionMapConfig";
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

const WORLD_MAP_KIND_LAYER: Record<string, number> = {
  rug: 0,
  sofa: 10,
  desk: 20,
  table: 25,
  stool: 30,
  chair: 35,
  computer: 40,
  monitor: 40,
  keyboard: 45,
  laptop: 45,
  lamp: 50,
  "book-stack": 50,
  mug: 50,
  plant: 55,
  cactus: 55,
};

function getWorldMapItemDepth(item: WorldMapItem) {
  return (WORLD_MAP_KIND_LAYER[item.kind] ?? 100) * 1000 + item.y;
}

function canUseWorldKeyboardInput() {
  return canUseGameKeyboardInput({
    disabled: usePanelStore.getState().isOpen,
  });
}
