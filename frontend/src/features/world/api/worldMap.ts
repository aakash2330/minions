import axios from "axios";
import { z } from "zod";

import { Direction } from "@/game/characters/characterConfig";
import { formatZodError } from "@/lib/zodError";

import type { WorldMapConfig, WorldMapItem } from "../map/types";

const ApiPointSchema = z.object({
  x: z.number(),
  y: z.number(),
});

const ApiWorkspaceElementSchema = z.object({
  id: z.string(),
  assignedSessionId: z.string().nullable(),
  kind: z.string(),
  label: z.string(),
  position: ApiPointSchema,
  facing: z.enum(Direction),
  assetId: z.string().nullable(),
  width: z.number().nullable(),
  height: z.number().nullable(),
});

const ApiWorkspaceElementsSchema = z.array(ApiWorkspaceElementSchema);

type ApiWorkspaceElement = z.infer<typeof ApiWorkspaceElementSchema>;

export function worldMapQueryKey(workspaceId?: string) {
  return ["world-map", workspaceId ?? "default"] as const;
}

export async function fetchWorldMapConfig(
  workspaceId: string,
): Promise<WorldMapConfig> {
  const response = await getWorkspaceElements(workspaceId);
  const result = ApiWorkspaceElementsSchema.safeParse(response);

  if (!result.success) {
    throw new Error(
      `Invalid workspace elements response: ${formatZodError(result.error)}`,
    );
  }

  return {
    version: 1,
    name: "AI Crew Studio",
    savedAt: null,
    canvas: {
      width: 1600,
      height: 900,
      background: "#edf0df",
      gridSize: 4,
    },
    items: result.data.flatMap(toWorldMapItem),
  };
}

function toWorldMapItem(element: ApiWorkspaceElement): WorldMapItem[] {
  if (!element.assetId || element.width === null || element.height === null) {
    return [];
  }

  return [
    {
      id: element.id,
      kind: element.kind,
      label: element.label,
      assetId: element.assetId,
      x: element.position.x,
      y: element.position.y,
      width: element.width,
      height: element.height,
      facing: element.facing,
    },
  ];
}

async function getWorkspaceElements(workspaceId: string) {
  try {
    const response = await axios.get<unknown>(
      `/api/workspaces/${encodeURIComponent(workspaceId)}/elements`,
    );
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        `Failed to load workspace elements: ${error.response?.status ?? error.message}`,
      );
    }

    throw error;
  }
}
