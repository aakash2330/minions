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
import mapConfigJson from "./map/mapConfig.json";
import type { WorldMapConfig } from "./map/types";

const MAP_CONFIG = mapConfigJson as WorldMapConfig;
const ASSET_BY_ID = new Map(
  WORLD_MAP_ASSETS.map((asset) => [asset.id, asset]),
);

type PhaserWorldProps = {
  sessions: SessionMapConfig[];
};

export function PhaserWorld({ sessions }: PhaserWorldProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const gameRef = useRef<Game | null>(null);
  const viewport = getWorldMapViewport(MAP_CONFIG);

  useEffect(() => {
    if (!parentRef.current || gameRef.current) {
      return;
    }

    gameRef.current = new Game({
      type: AUTO,
      parent: parentRef.current,
      transparent: true,
      width: MAP_CONFIG.canvas.width,
      height: MAP_CONFIG.canvas.height,
      render: {
        pixelArt: true,
      },
      scene: [
        new WorldScene({
          canUseKeyboardInput: canUseWorldKeyboardInput,
          mapConfig: MAP_CONFIG,
          sessions,
        }),
      ],
    });

    return () => {
      gameRef.current?.destroy(true);
      gameRef.current = null;
    };
  }, [sessions, viewport.height, viewport.width]);

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
          data-studio-scene={MAP_CONFIG.name === "AI Crew Studio"}
          style={
            {
              "--world-map-grid-size": `${MAP_CONFIG.canvas.gridSize}px`,
              backgroundColor: MAP_CONFIG.canvas.background,
              height: MAP_CONFIG.canvas.height,
              transform: `scale(${WORLD_MAP_VIEW_SCALE})`,
              width: MAP_CONFIG.canvas.width,
            } as CSSProperties
          }
        >
          {MAP_CONFIG.items.map((item) => {
            const asset = ASSET_BY_ID.get(item.assetId);

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
                  transform: `rotate(${item.rotation}deg) scaleX(${item.flipX ? -1 : 1})`,
                  width: item.width,
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

function canUseWorldKeyboardInput() {
  return canUseGameKeyboardInput({
    disabled: usePanelStore.getState().isOpen,
  });
}
