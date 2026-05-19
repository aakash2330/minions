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
          usePanelStore.getState().open({
            type: PanelContentType.SessionChat,
            sessionId: event.sessionId,
          });
          toast(event.type);
          break;

        case WsEventType.ApprovalResolved:
          queryClient.setQueryData<SessionApprovalRequestState>(
            sessionApprovalRequestQueryKey(event.sessionId),
            null,
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
