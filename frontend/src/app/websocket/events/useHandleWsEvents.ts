import { useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";

import {
  sessionQueryKey,
  SessionMessageRole,
  SessionMessageStatus,
  type Session,
  type SessionMessage,
} from "@/features/sessions/api/sessions";

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
          break;

        case WsEventType.ApprovalRequest:
          toast(event.type);
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
