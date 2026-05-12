import axios from "axios";

import { formatZodError } from "@/lib/zodError";

import {
  ApiMinionSchema,
  ApiMinionsResponseSchema,
  ApiWorkspaceElementsSchema,
  ApiWorkspaceResponseSchema,
  ApiWorkspacesResponseSchema,
  type ApiWorkspace,
} from "./minionSchemas";
import { getAssignedElementsByMinionId, toMinion } from "./minionMappers";
import type { Minion } from "./minionMappers";

export { MinionMessageRole } from "./minionMappers";
export type { Minion, MinionMessage } from "./minionMappers";

export function minionsQueryKey(workspaceId?: string) {
  return ["minions", workspaceId ?? "all"] as const;
}

export function minionQueryKey(minionId: string) {
  return ["minions", "detail", minionId] as const;
}

export async function fetchMinions(workspaceId?: string): Promise<Minion[]> {
  const [minionsResponse, workspacesResponse] = await Promise.all([
    getApiMinions(workspaceId),
    getWorkspaces(),
  ]);
  const minionsResult = ApiMinionsResponseSchema.safeParse(minionsResponse);
  const workspacesResult =
    ApiWorkspacesResponseSchema.safeParse(workspacesResponse);

  if (!minionsResult.success) {
    throw new Error(
      `Invalid minions response: ${formatZodError(minionsResult.error)}`,
    );
  }

  if (!workspacesResult.success) {
    throw new Error(
      `Invalid workspaces response: ${formatZodError(workspacesResult.error)}`,
    );
  }

  const workspaces = workspaceId
    ? workspacesResult.data.filter((workspace) => workspace.id === workspaceId)
    : workspacesResult.data;
  const workspaceElements = await fetchElementsByWorkspaceId(workspaces);
  const workspaceById = new Map(
    workspacesResult.data.map((workspace) => [workspace.id, workspace]),
  );
  const elementsByMinionId = getAssignedElementsByMinionId(workspaceElements);

  return minionsResult.data.map((minion) =>
    toMinion(
      minion,
      workspaceById.get(minion.workspaceId),
      elementsByMinionId.get(minion.minionId) ?? [],
    ),
  );
}

export async function fetchMinion(minionId: string): Promise<Minion> {
  const minionResponse = await getApiMinion(minionId);
  const minionResult = ApiMinionSchema.safeParse(minionResponse);

  if (!minionResult.success) {
    throw new Error(
      `Invalid minion response: ${formatZodError(minionResult.error)}`,
    );
  }

  const [workspaceResponse, workspaceElementsResponse] = await Promise.all([
    getWorkspace(minionResult.data.workspaceId),
    getWorkspaceElements(minionResult.data.workspaceId),
  ]);
  const workspaceResult =
    ApiWorkspaceResponseSchema.safeParse(workspaceResponse);
  const workspaceElementsResult = ApiWorkspaceElementsSchema.safeParse(
    workspaceElementsResponse,
  );

  if (!workspaceResult.success) {
    throw new Error(
      `Invalid workspace response: ${formatZodError(workspaceResult.error)}`,
    );
  }

  if (!workspaceElementsResult.success) {
    throw new Error(
      `Invalid workspace elements response: ${formatZodError(workspaceElementsResult.error)}`,
    );
  }

  return toMinion(
    minionResult.data,
    workspaceResult.data,
    workspaceElementsResult.data.filter(
      (element) => element.assignedMinionId === minionId,
    ),
  );
}

async function getApiMinions(workspaceId?: string) {
  const path = workspaceId
    ? `/api/workspaces/${encodeURIComponent(workspaceId)}/sessions`
    : "/api/sessions";

  try {
    const response = await axios.get<unknown>(path);
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        `Failed to load minions: ${error.response?.status ?? error.message}`,
      );
    }

    throw error;
  }
}

async function getApiMinion(minionId: string) {
  try {
    const response = await axios.get<unknown>(
      `/api/sessions/${encodeURIComponent(minionId)}`,
    );
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        `Failed to load minion: ${error.response?.status ?? error.message}`,
      );
    }

    throw error;
  }
}

async function getWorkspaces() {
  try {
    const response = await axios.get<unknown>("/api/workspaces");
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        `Failed to load workspaces: ${error.response?.status ?? error.message}`,
      );
    }

    throw error;
  }
}

async function getWorkspace(workspaceId: string) {
  try {
    const response = await axios.get<unknown>(
      `/api/workspaces/${encodeURIComponent(workspaceId)}`,
    );
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        `Failed to load workspace: ${error.response?.status ?? error.message}`,
      );
    }

    throw error;
  }
}

async function fetchElementsByWorkspaceId(workspaces: ApiWorkspace[]) {
  const entries = await Promise.all(
    workspaces.map(async (workspace) => {
      const result = ApiWorkspaceElementsSchema.safeParse(
        await getWorkspaceElements(workspace.id),
      );

      if (!result.success) {
        throw new Error(
          `Invalid workspace elements response: ${formatZodError(result.error)}`,
        );
      }

      return [workspace.id, result.data] as const;
    }),
  );

  return new Map(entries);
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
