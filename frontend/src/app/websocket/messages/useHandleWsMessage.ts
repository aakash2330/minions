import { useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";

import {
  sessionQueryKey,
  SessionMessageRole,
  SessionMessageStatus,
  type Session,
  type SessionMessage,
} from "@/features/sessions/api/sessions";

import { type WsMessage, WsMessageType } from "./wsMessage";

export function useHandleWsMessage() {
  const queryClient = useQueryClient();

  const handleWsMessage = useCallback(
    (message: WsMessage): WsMessage => {
      switch (message.type) {
        case WsMessageType.TurnStart:
          queryClient.setQueryData<Session>(
            sessionQueryKey(message.sessionId),
            (session) => {
              if (!session) return session;

              return {
                ...session,
                messages: appendUserMessage(
                  session.messages,
                  message.sessionId,
                  message.prompt,
                ),
              };
            },
          );
          break;

        case WsMessageType.ApprovalResponse:
          break;
      }

      return message;
    },
    [queryClient],
  );

  return { handleWsMessage };
}

function appendUserMessage(
  messages: SessionMessage[],
  sessionId: string,
  prompt: string,
) {
  return [
    ...messages,
    {
      id: crypto.randomUUID(),
      sessionId,
      role: SessionMessageRole.User,
      status: SessionMessageStatus.Complete,
      text: prompt,
    },
  ];
}
