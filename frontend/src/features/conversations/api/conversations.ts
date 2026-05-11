import { z } from "zod";

import type { SessionId } from "@/game/minionMapConfig";
import { formatZodError } from "@/lib/zodError";

export enum ConversationMessageRole {
  Assistant = "assistant",
  System = "system",
  User = "user",
}

export type HistoricalConversationMessage = {
  id: string;
  sessionId: string;
  role: ConversationMessageRole | string;
  status: string;
  text: string;
};

export type HistoricalConversation = {
  id: string;
  sessionId: SessionId;
  minionName: string | null;
  messages: HistoricalConversationMessage[];
  status: string;
  title: string;
  workspaceId: string;
  workspaceRootPath: string | null;
};

const ApiSessionMessageSchema = z.object({
  id: z.string(),
  session_id: z.string(),
  role: z.string(),
  text: z.string(),
  status: z.string(),
});

const ApiSessionSchema = z.object({
  session_id: z.string(),
  workspaceId: z.string(),
  name: z.string(),
  kind: z.string(),
  status: z.string(),
  messages: z.array(ApiSessionMessageSchema),
});

const ApiWorkspaceSchema = z.object({
  id: z.string(),
  name: z.string(),
  rootPath: z.string().nullable(),
});

const ApiDataResponseSchema = z.object({
  workspaces: z.array(ApiWorkspaceSchema),
  sessions: z.array(ApiSessionSchema),
});

type ApiSession = z.infer<typeof ApiSessionSchema>;
type ApiSessionMessage = z.infer<typeof ApiSessionMessageSchema>;
type ApiWorkspace = z.infer<typeof ApiWorkspaceSchema>;

export const historicalConversationsQueryKey = [
  "historical-conversations",
] as const;

export async function fetchHistoricalConversations(): Promise<
  HistoricalConversation[]
> {
  const response = await fetch("/api/data");

  if (!response.ok) {
    throw new Error(`Failed to load conversations: ${response.status}`);
  }

  const result = ApiDataResponseSchema.safeParse(await response.json());

  if (!result.success) {
    throw new Error(
      `Invalid conversations response: ${formatZodError(result.error)}`,
    );
  }

  const workspaceById = new Map(
    result.data.workspaces.map((workspace) => [workspace.id, workspace]),
  );

  return result.data.sessions.map((session) =>
    toHistoricalConversation(session, workspaceById.get(session.workspaceId)),
  );
}

function toHistoricalConversation(
  session: ApiSession,
  workspace: ApiWorkspace | undefined,
): HistoricalConversation {
  return {
    id: session.session_id,
    sessionId: session.session_id,
    minionName: session.name,
    messages: session.messages.map(toHistoricalConversationMessage),
    status: session.status,
    title: session.name,
    workspaceId: session.workspaceId,
    workspaceRootPath: workspace?.rootPath ?? null,
  };
}

function toHistoricalConversationMessage(
  message: ApiSessionMessage,
): HistoricalConversationMessage {
  return {
    id: message.id,
    sessionId: message.session_id,
    role: toConversationMessageRole(message.role),
    status: message.status,
    text: message.text,
  };
}

function toConversationMessageRole(role: string) {
  if (
    Object.values(ConversationMessageRole).includes(
      role as ConversationMessageRole,
    )
  ) {
    return role as ConversationMessageRole;
  }

  return role;
}
