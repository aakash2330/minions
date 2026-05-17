export type WorldMapAsset = {
  id: string;
  name: string;
  fileName: string;
  category: string;
  path: string;
  url: string;
  width: number;
  height: number;
  tags: string[];
};

export type WorldMapPoint = {
  x: number;
  y: number;
};

export type WorldMapItem = {
  id: string;
  assetId: string;
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number;
  flipX: boolean;
};

export type WorldMapConfig = {
  version: number;
  name: string;
  savedAt: string | null;
  canvas: {
    width: number;
    height: number;
    background: string;
    gridSize: number;
  };
  items: WorldMapItem[];
};
