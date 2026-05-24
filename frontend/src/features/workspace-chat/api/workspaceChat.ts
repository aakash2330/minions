import axios from "axios";

import { formatZodError } from "@/lib/zodError";

import {
  ApiWorkspaceChatMessagesResponseSchema,
  type ApiWorkspaceChatMessage,
} from "./workspaceChatSchemas";
import {
  toWorkspaceChatMessage,
  type WorkspaceChatMessage,
} from "./workspaceChatMappers";

export {
  WorkspaceChatMessageRole,
  WorkspaceChatMessageStatus,
} from "./workspaceChatMappers";
export type { WorkspaceChatMessage } from "./workspaceChatMappers";

export type WorkspaceChatApprovalRequestState = {
  sessionId: string;
  status: "pending" | "responding";
} | null;

export function workspaceChatMessagesQueryKey(workspaceId: string) {
  return ["workspace-chat", workspaceId, "messages"] as const;
}

export function workspaceChatApprovalRequestQueryKey(workspaceId: string) {
  return ["workspace-chat", workspaceId, "approval"] as const;
}

export async function fetchWorkspaceChatMessages(
  workspaceId: string,
): Promise<WorkspaceChatMessage[]> {
  const response = await getWorkspaceChatMessages(workspaceId);
  const result = ApiWorkspaceChatMessagesResponseSchema.safeParse(response);

  if (!result.success) {
    throw new Error(
      `Invalid workspace chat response: ${formatZodError(result.error)}`,
    );
  }

  return result.data.map(toWorkspaceChatMessage);
}

async function getWorkspaceChatMessages(workspaceId: string) {
  try {
    const response = await axios.get<ApiWorkspaceChatMessage[]>(
      `/api/workspaces/${encodeURIComponent(workspaceId)}/chat/messages`,
    );
    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        `Failed to load workspace chat: ${error.response?.status ?? error.message}`,
      );
    }

    throw error;
  }
}
