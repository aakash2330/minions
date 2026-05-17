import axios from "axios";

import {
  ApiWorkspacesResponseSchema,
  type ApiWorkspace,
} from "@/features/sessions/api/sessionSchemas";
import { formatZodError } from "@/lib/zodError";

export type Workspace = ApiWorkspace;

export function workspacesQueryKey() {
  return ["workspaces"] as const;
}

export async function fetchWorkspaces(): Promise<Workspace[]> {
  try {
    const response = await axios.get<unknown>("/api/workspaces");
    const result = ApiWorkspacesResponseSchema.safeParse(response.data);

    if (!result.success) {
      throw new Error(
        `Invalid workspaces response: ${formatZodError(result.error)}`,
      );
    }

    return result.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      throw new Error(
        `Failed to load workspaces: ${error.response?.status ?? error.message}`,
      );
    }

    throw error;
  }
}
