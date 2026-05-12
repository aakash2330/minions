import axios from "axios";

import { formatZodError } from "@/lib/zodError";

import {
  ApiSessionSchema,
  ApiSessionsResponseSchema,
  ApiWorkspaceElementsSchema,
  ApiWorkspaceResponseSchema,
  ApiWorkspacesResponseSchema,
  type ApiWorkspace,
} from "./sessionSchemas";
import { getAssignedElementsBySessionId, toSession } from "./sessionMappers";
import type { Session } from "./sessionMappers";

export { SessionMessageRole } from "./sessionMappers";
export type { Session, SessionMessage } from "./sessionMappers";

export function sessionsQueryKey(workspaceId?: string) {
  return ["sessions", workspaceId ?? "all"] as const;
}

export function sessionQueryKey(sessionId: string) {
  return ["sessions", "detail", sessionId] as const;
}

export async function fetchSessions(workspaceId?: string): Promise<Session[]> {
  const [sessionsResponse, workspacesResponse] = await Promise.all([
    getApiSessions(workspaceId),
    getWorkspaces(),
  ]);
  const sessionsResult = ApiSessionsResponseSchema.safeParse(sessionsResponse);
  const workspacesResult =
    ApiWorkspacesResponseSchema.safeParse(workspacesResponse);

  if (!sessionsResult.success) {
    throw new Error(
      `Invalid sessions response: ${formatZodError(sessionsResult.error)}`,
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
  const elementsBySessionId = getAssignedElementsBySessionId(workspaceElements);

  return sessionsResult.data.map((session) =>
    toSession(
      session,
      workspaceById.get(session.workspaceId),
      elementsBySessionId.get(session.sessionId) ?? [],
    ),
  );
}

export async function fetchSession(sessionId: string): Promise<Session> {
  const sessionResponse = await getApiSession(sessionId);
  const sessionResult = ApiSessionSchema.safeParse(sessionResponse);

  if (!sessionResult.success) {
    throw new Error(
      `Invalid session response: ${formatZodError(sessionResult.error)}`,
    );
  }

  const [workspaceResponse, workspaceElementsResponse] = await Promise.all([
    getWorkspace(sessionResult.data.workspaceId),
    getWorkspaceElements(sessionResult.data.workspaceId),
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

  return toSession(
    sessionResult.data,
    workspaceResult.data,
    workspaceElementsResult.data.filter(
      (element) => element.assignedSessionId === sessionId,
    ),
  );
}

async function getApiSessions(workspaceId?: string) {
  const path = workspaceId
    ? `/api/workspaces/${encodeURIComponent(workspaceId)}/sessions`
    : "/api/sessions";

  try {
    const response = await axios.get<unknown>(path);
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        `Failed to load sessions: ${error.response?.status ?? error.message}`,
      );
    }

    throw error;
  }
}

async function getApiSession(sessionId: string) {
  try {
    const response = await axios.get<unknown>(
      `/api/sessions/${encodeURIComponent(sessionId)}`,
    );
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        `Failed to load session: ${error.response?.status ?? error.message}`,
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
