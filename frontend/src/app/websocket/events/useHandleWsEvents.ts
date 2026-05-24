import { useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";

import {
  sessionQueryKey,
  SessionMessageRole,
  SessionMessageStatus,
  type Session,
  type SessionMessage,
  sessionApprovalRequestQueryKey,
  type SessionApprovalRequestState,
} from "@/features/sessions/api/sessions";
import {
  WorkspaceChatMessageRole,
  WorkspaceChatMessageStatus,
  workspaceChatApprovalRequestQueryKey,
  workspaceChatMessagesQueryKey,
  type WorkspaceChatApprovalRequestState,
  type WorkspaceChatMessage,
} from "@/features/workspace-chat/api/workspaceChat";
import {
  PanelContentType,
  usePanelStore,
} from "@/features/panel/stores/panelStore";
import { dispatchSessionInteraction } from "@/game/WorldScene";

import { type WsEvent, WsEventSchema, WsEventType } from "./wsEvent";
import { toast } from "sonner";

export function useHandleWsEvents() {
  const queryClient = useQueryClient();

  const handleWsEvent = useCallback(
    (messageEvent: MessageEvent<string>) => {
      const parsedEvent = WsEventSchema.safeParse(
        JSON.parse(messageEvent.data),
      );

      if (!parsedEvent.success) {
        throw new Error("couldn't parse event");
      }

      const event: WsEvent = parsedEvent.data;
      console.log("event from websocket", { event });

      switch (event.type) {
        case WsEventType.TurnStarted:
          queryClient.invalidateQueries({
            queryKey: sessionQueryKey(event.sessionId),
          });
          break;

        case WsEventType.AssistantDelta:
          queryClient.setQueryData<Session>(
            sessionQueryKey(event.sessionId),
            (session) => {
              if (!session) return session;

              return {
                ...session,
                messages: appendAssistantDelta(
                  session.messages,
                  event.sessionId,
                  event.messageId,
                  event.text,
                ),
              };
            },
          );
          break;

        case WsEventType.TurnCompleted:
          toast(event.type);
          queryClient.invalidateQueries({
            queryKey: sessionQueryKey(event.sessionId),
          });
          queryClient.setQueryData<SessionApprovalRequestState>(
            sessionApprovalRequestQueryKey(event.sessionId),
            null,
          );
          break;

        case WsEventType.ApprovalRequest:
          queryClient.setQueryData<SessionApprovalRequestState>(
            sessionApprovalRequestQueryKey(event.sessionId),
            "pending",
          );
          if (event.workspaceId) {
            queryClient.setQueryData<WorkspaceChatApprovalRequestState>(
              workspaceChatApprovalRequestQueryKey(event.workspaceId),
              { sessionId: event.sessionId, status: "pending" },
            );
          }
          openApprovalPanel(event.sessionId, event.workspaceId);
          toast(event.type);
          break;

        case WsEventType.ApprovalResolved:
          queryClient.setQueryData<SessionApprovalRequestState>(
            sessionApprovalRequestQueryKey(event.sessionId),
            null,
          );
          if (event.workspaceId) {
            queryClient.setQueryData<WorkspaceChatApprovalRequestState>(
              workspaceChatApprovalRequestQueryKey(event.workspaceId),
              null,
            );
          }
          break;

        case WsEventType.WorkspaceChatMessageCreated:
          queryClient.setQueryData<WorkspaceChatMessage[]>(
            workspaceChatMessagesQueryKey(event.workspaceId),
            (messages) =>
              appendWorkspaceChatMessage(messages ?? [], {
                id: event.messageId,
                workspaceId: event.workspaceId,
                sessionId: event.sessionId,
                sessionMessageId: null,
                parentMessageId: null,
                role: toWorkspaceChatMessageRole(event.role),
                status: toWorkspaceChatMessageStatus(event.status),
                text: event.text,
              }),
          );
          break;

        case WsEventType.WorkspaceChatMessageDelta:
          queryClient.setQueryData<WorkspaceChatMessage[]>(
            workspaceChatMessagesQueryKey(event.workspaceId),
            (messages) =>
              appendWorkspaceChatMessageDelta(
                messages ?? [],
                event.workspaceId,
                event.messageId,
                event.sessionId,
                event.text,
              ),
          );
          break;

        case WsEventType.WorkspaceChatMessageCompleted:
          queryClient.setQueryData<WorkspaceChatMessage[]>(
            workspaceChatMessagesQueryKey(event.workspaceId),
            (messages) =>
              completeWorkspaceChatMessage(
                messages ?? [],
                event.workspaceId,
                event.messageId,
                event.sessionId,
                event.status,
                event.text,
              ),
          );
          break;

        case WsEventType.SessionInteraction:
          dispatchSessionInteraction({
            sessionId: event.sessionId,
            interactionType: event.interactionType,
          });
          break;

        case WsEventType.Error:
          if (event.sessionId) {
            queryClient.invalidateQueries({
              queryKey: sessionQueryKey(event.sessionId),
            });
            queryClient.setQueryData<SessionApprovalRequestState>(
              sessionApprovalRequestQueryKey(event.sessionId),
              null,
            );
          }
          toast(event.message);
          break;

        default:
          console.log("unhandled event from websocket", { event });
      }
    },
    [queryClient],
  );

  return { handleWsEvent };
}

function openApprovalPanel(sessionId: string, workspaceId?: string) {
  const panelStore = usePanelStore.getState();
  const content = panelStore.content;

  if (
    workspaceId &&
    content?.type === PanelContentType.GlobalChat &&
    content.workspaceId === workspaceId
  ) {
    return;
  }

  panelStore.open({
    type: PanelContentType.SessionChat,
    sessionId,
  });
}

function appendAssistantDelta(
  messages: SessionMessage[],
  sessionId: string,
  messageId: string,
  text: string,
) {
  const messageIndex = messages.findIndex(
    (message) => message.id === messageId,
  );

  if (messageIndex === -1) {
    return [
      ...messages,
      {
        id: messageId,
        sessionId,
        role: SessionMessageRole.Assistant,
        status: SessionMessageStatus.Streaming,
        text,
      },
    ];
  }

  return messages.map((message, index) =>
    index === messageIndex
      ? {
          ...message,
          text: message.text + text,
        }
      : message,
  );
}

function appendWorkspaceChatMessage(
  messages: WorkspaceChatMessage[],
  message: WorkspaceChatMessage,
) {
  if (messages.some((existingMessage) => existingMessage.id === message.id)) {
    return messages;
  }

  return [...messages, message];
}

function appendWorkspaceChatMessageDelta(
  messages: WorkspaceChatMessage[],
  workspaceId: string,
  messageId: string,
  sessionId: string | null,
  text: string,
) {
  const messageIndex = messages.findIndex((message) => message.id === messageId);

  if (messageIndex === -1) {
    return [
      ...messages,
      {
        id: messageId,
        workspaceId,
        sessionId,
        sessionMessageId: null,
        parentMessageId: null,
        role: WorkspaceChatMessageRole.Assistant,
        status: WorkspaceChatMessageStatus.Streaming,
        text,
      },
    ];
  }

  return messages.map((message, index) =>
    index === messageIndex
      ? {
          ...message,
          text: message.text + text,
          status: WorkspaceChatMessageStatus.Streaming,
        }
      : message,
  );
}

function completeWorkspaceChatMessage(
  messages: WorkspaceChatMessage[],
  workspaceId: string,
  messageId: string,
  sessionId: string | null,
  status: string,
  text?: string,
) {
  const messageIndex = messages.findIndex((message) => message.id === messageId);

  if (messageIndex === -1) {
    return [
      ...messages,
      {
        id: messageId,
        workspaceId,
        sessionId,
        sessionMessageId: null,
        parentMessageId: null,
        role: WorkspaceChatMessageRole.Assistant,
        status: toWorkspaceChatMessageStatus(status),
        text: "",
      },
    ];
  }

  return messages.map((message, index) =>
    index === messageIndex
      ? {
          ...message,
          status: toWorkspaceChatMessageStatus(status),
          ...(text === undefined ? {} : { text }),
        }
      : message,
  );
}

function toWorkspaceChatMessageRole(role: string): WorkspaceChatMessageRole {
  switch (role) {
    case WorkspaceChatMessageRole.Assistant:
      return WorkspaceChatMessageRole.Assistant;
    case WorkspaceChatMessageRole.System:
      return WorkspaceChatMessageRole.System;
    case WorkspaceChatMessageRole.User:
    default:
      return WorkspaceChatMessageRole.User;
  }
}

function toWorkspaceChatMessageStatus(status: string): WorkspaceChatMessageStatus {
  switch (status) {
    case WorkspaceChatMessageStatus.Pending:
      return WorkspaceChatMessageStatus.Pending;
    case WorkspaceChatMessageStatus.Streaming:
      return WorkspaceChatMessageStatus.Streaming;
    case WorkspaceChatMessageStatus.Error:
      return WorkspaceChatMessageStatus.Error;
    case WorkspaceChatMessageStatus.Complete:
    default:
      return WorkspaceChatMessageStatus.Complete;
  }
}
