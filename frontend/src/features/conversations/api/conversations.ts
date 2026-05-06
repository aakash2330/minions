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
  threadId: string;
  role: ConversationMessageRole | string;
  status: string;
  text: string;
};

export type HistoricalConversation = {
  id: string;
  cwd: string | null;
  sessionId: SessionId | null;
  minionName: string | null;
  messages: HistoricalConversationMessage[];
  status: string;
  title: string;
  workspaceId: string;
};

const ApiThreadMessageSchema = z.object({
  id: z.string(),
  thread_id: z.string(),
  role: z.string(),
  text: z.string(),
  status: z.string(),
});

const ApiThreadSchema = z.object({
  id: z.string(),
  workspaceId: z.string(),
  session_id: z.string().nullable(),
  title: z.string(),
  cwd: z.string().nullable(),
  status: z.string(),
  messages: z.array(ApiThreadMessageSchema),
});

const ApiSessionSchema = z.object({
  session_id: z.string(),
  name: z.string(),
});

const ApiDataResponseSchema = z.object({
  sessions: z.array(ApiSessionSchema),
  threads: z.array(ApiThreadSchema),
});

type ApiThread = z.infer<typeof ApiThreadSchema>;
type ApiThreadMessage = z.infer<typeof ApiThreadMessageSchema>;

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

  const data = result.data;
  const minionNames = new Map(
    data.sessions.map((session) => [session.session_id, session.name]),
  );

  return data.threads.map((thread) =>
    toHistoricalConversation(thread, minionNames),
  );
}

function toHistoricalConversation(
  thread: ApiThread,
  minionNames: Map<SessionId, string>,
): HistoricalConversation {
  const minionName = thread.session_id
    ? (minionNames.get(thread.session_id) ?? thread.session_id)
    : null;

  return {
    id: thread.id,
    cwd: thread.cwd,
    sessionId: thread.session_id,
    minionName,
    messages: thread.messages.map(toHistoricalConversationMessage),
    status: thread.status,
    title: thread.title,
    workspaceId: thread.workspaceId,
  };
}

function toHistoricalConversationMessage(
  message: ApiThreadMessage,
): HistoricalConversationMessage {
  return {
    id: message.id,
    threadId: message.thread_id,
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
