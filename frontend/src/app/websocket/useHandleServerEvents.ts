import { useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";

import {
  sessionQueryKey,
  SessionMessageRole,
  type Session,
  type SessionMessage,
} from "@/features/sessions/api/sessions";

import { ServerEvent, ServerEventSchema, ServerEventType } from "./serverEvent";
import { toast } from "sonner";

export function useHandleServerEvents() {
  const queryClient = useQueryClient();

  const handleServerEvent = useCallback(
    (messageEvent: MessageEvent<string>) => {
      const parsedMessage = ServerEventSchema.safeParse(
        JSON.parse(messageEvent.data),
      );

      if (!parsedMessage.success) {
        throw new Error("couldn't parse server event");
      }

      const message: ServerEvent = parsedMessage.data;
      console.log("event from server", { message });

      switch (message.type) {
        case ServerEventType.TurnStarted:
          break;

        case ServerEventType.AssistantDelta:
          queryClient.setQueryData<Session>(
            sessionQueryKey(message.sessionId),
            (session) => {
              if (!session) return session;

              return {
                ...session,
                messages: appendAssistantDelta(
                  session.messages,
                  message.sessionId,
                  message.messageId,
                  message.text,
                ),
              };
            },
          );
          break;

        case ServerEventType.TurnCompleted:
          toast(message.type);
          queryClient.invalidateQueries({
            queryKey: sessionQueryKey(message.sessionId),
          });
          break;

        case ServerEventType.ApprovalRequest:
          toast(message.type);
          break;

        default:
          console.log("unhandled event from server", { message });
      }
    },
    [queryClient],
  );

  return { handleServerEvent };
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
        status: "streaming",
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
