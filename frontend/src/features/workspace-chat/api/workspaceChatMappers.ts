import {
  ApiWorkspaceChatMessageRole,
  ApiWorkspaceChatMessageStatus,
  type ApiWorkspaceChatMessage,
} from "./workspaceChatSchemas";

export enum WorkspaceChatMessageRole {
  Assistant = "assistant",
  System = "system",
  User = "user",
}

export enum WorkspaceChatMessageStatus {
  Pending = "pending",
  Streaming = "streaming",
  Complete = "complete",
  Error = "error",
}

export type WorkspaceChatMessage = {
  id: string;
  workspaceId: string;
  sessionId: string | null;
  sessionMessageId: string | null;
  parentMessageId: string | null;
  role: WorkspaceChatMessageRole;
  text: string;
  status: WorkspaceChatMessageStatus;
};

export function toWorkspaceChatMessage(
  message: ApiWorkspaceChatMessage,
): WorkspaceChatMessage {
  return {
    id: message.id,
    workspaceId: message.workspaceId,
    sessionId: message.sessionId,
    sessionMessageId: message.sessionMessageId,
    parentMessageId: message.parentMessageId,
    role: toWorkspaceChatMessageRole(message.role),
    text: message.text,
    status: toWorkspaceChatMessageStatus(message.status),
  };
}

function toWorkspaceChatMessageRole(
  role: ApiWorkspaceChatMessage["role"],
): WorkspaceChatMessageRole {
  switch (role) {
    case ApiWorkspaceChatMessageRole.Assistant:
      return WorkspaceChatMessageRole.Assistant;
    case ApiWorkspaceChatMessageRole.System:
      return WorkspaceChatMessageRole.System;
    case ApiWorkspaceChatMessageRole.User:
      return WorkspaceChatMessageRole.User;
  }
}

function toWorkspaceChatMessageStatus(
  status: ApiWorkspaceChatMessage["status"],
): WorkspaceChatMessageStatus {
  switch (status) {
    case ApiWorkspaceChatMessageStatus.Pending:
      return WorkspaceChatMessageStatus.Pending;
    case ApiWorkspaceChatMessageStatus.Streaming:
      return WorkspaceChatMessageStatus.Streaming;
    case ApiWorkspaceChatMessageStatus.Complete:
      return WorkspaceChatMessageStatus.Complete;
    case ApiWorkspaceChatMessageStatus.Error:
      return WorkspaceChatMessageStatus.Error;
  }
}
