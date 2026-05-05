import type { MinionId } from "@/game/minionMapConfig";

export enum ConversationMessageRole {
  Assistant = "assistant",
  System = "system",
  User = "user",
}

export type HistoricalConversationMessage = {
  id: string;
  conversationId: string;
  role: ConversationMessageRole | string;
  status: string;
  text: string;
};

export type HistoricalConversation = {
  id: string;
  cwd: string | null;
  currentSessionId: string | null;
  minionId: MinionId | null;
  minionName: string | null;
  messages: HistoricalConversationMessage[];
  status: string;
  title: string;
  workspaceId: string;
};

type ApiDataResponse = {
  conversations: ApiConversation[];
  minions: ApiMinion[];
};

type ApiConversation = {
  id: string;
  workspaceId: string;
  minionId: MinionId | null;
  title: string;
  currentSessionId: string | null;
  cwd: string | null;
  status: string;
  messages: ApiConversationMessage[];
};

type ApiConversationMessage = {
  id: string;
  conversationId: string;
  role: string;
  text: string;
  status: string;
};

type ApiMinion = {
  id: MinionId;
  name: string;
};

export async function fetchHistoricalConversations(): Promise<
  HistoricalConversation[]
> {
  const response = await fetch("/api/data");

  if (!response.ok) {
    throw new Error(`Failed to load conversations: ${response.status}`);
  }

  const data = (await response.json()) as ApiDataResponse;
  const minionNames = new Map(
    data.minions.map((minion) => [minion.id, minion.name]),
  );

  return data.conversations.map((conversation) =>
    toHistoricalConversation(conversation, minionNames),
  );
}

function toHistoricalConversation(
  conversation: ApiConversation,
  minionNames: Map<MinionId, string>,
): HistoricalConversation {
  const minionName = conversation.minionId
    ? (minionNames.get(conversation.minionId) ?? conversation.minionId)
    : null;

  return {
    id: conversation.id,
    cwd: conversation.cwd,
    currentSessionId: conversation.currentSessionId,
    minionId: conversation.minionId,
    minionName,
    messages: conversation.messages.map(toHistoricalConversationMessage),
    status: conversation.status,
    title: conversation.title,
    workspaceId: conversation.workspaceId,
  };
}

function toHistoricalConversationMessage(
  message: ApiConversationMessage,
): HistoricalConversationMessage {
  return {
    id: message.id,
    conversationId: message.conversationId,
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
