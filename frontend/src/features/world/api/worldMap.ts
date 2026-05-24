import axios from "axios";
import { z } from "zod";

import { Direction } from "@/game/characters/characterConfig";
import { formatZodError } from "@/lib/zodError";

import { WORLD_MAP_ASSETS } from "../map/assets.generated";
import type { WorldMapConfig, WorldMapItem } from "../map/types";

const ASSET_BY_ID = new Map(WORLD_MAP_ASSETS.map((asset) => [asset.id, asset]));
const ASSET_BY_FILE_NAME = new Map(
  WORLD_MAP_ASSETS.map((asset) => [asset.fileName, asset]),
);

const ApiWorldMapItemSchema = z.object({
  id: z.string(),
  kind: z.string(),
  label: z.string(),
  assetId: z.string(),
  x: z.number(),
  y: z.number(),
  width: z.number(),
  height: z.number(),
  facing: z.enum(Direction),
});

const ApiWorldMapConfigSchema = z.object({
  version: z.number(),
  name: z.string(),
  savedAt: z.string().nullable(),
  canvas: z.object({
    width: z.number(),
    height: z.number(),
    background: z.string(),
    gridSize: z.number(),
  }),
  items: z.array(ApiWorldMapItemSchema),
});

type ApiWorldMapItem = z.infer<typeof ApiWorldMapItemSchema>;

export function worldMapQueryKey(workspaceId?: string) {
  return ["world-map", workspaceId ?? "default"] as const;
}

export async function fetchWorldMapConfig(
  workspaceId: string,
): Promise<WorldMapConfig> {
  const response = await getWorkspaceMapConfig(workspaceId);
  const result = ApiWorldMapConfigSchema.safeParse(response);

  if (!result.success) {
    throw new Error(
      `Invalid workspace map config response: ${formatZodError(result.error)}`,
    );
  }

  return {
    ...result.data,
    items: result.data.items.flatMap(toWorldMapItem),
  };
}

function toWorldMapItem(item: ApiWorldMapItem): WorldMapItem[] {
  const asset =
    ASSET_BY_ID.get(item.assetId) ?? ASSET_BY_FILE_NAME.get(item.assetId);

  if (!asset) {
    return [];
  }

  return [
    {
      id: item.id,
      kind: item.kind,
      label: item.label,
      assetId: asset.id,
      x: item.x,
      y: item.y,
      width: item.width,
      height: item.height,
      facing: item.facing,
    },
  ];
}

async function getWorkspaceMapConfig(workspaceId: string) {
  try {
    const response = await axios.get<unknown>(
      `/api/workspaces/${encodeURIComponent(workspaceId)}/map-config`,
    );
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        `Failed to load workspace map config: ${error.response?.status ?? error.message}`,
      );
    }

    throw error;
  }
}
