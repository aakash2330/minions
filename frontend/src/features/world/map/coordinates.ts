import type { WorldMapConfig, WorldMapPoint } from "./types";

export const WORLD_MAP_VIEW_SCALE = 0.7;

export function getWorldMapViewport(mapConfig: WorldMapConfig) {
  return {
    width: Math.round(mapConfig.canvas.width * WORLD_MAP_VIEW_SCALE),
    height: Math.round(mapConfig.canvas.height * WORLD_MAP_VIEW_SCALE),
  };
}

export function getWorldMapCenter(mapConfig: WorldMapConfig): WorldMapPoint {
  return {
    x: mapConfig.canvas.width / 2,
    y: mapConfig.canvas.height / 2,
  };
}
