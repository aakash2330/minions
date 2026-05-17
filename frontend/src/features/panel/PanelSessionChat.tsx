import { SessionChat } from "@/features/session-chat/SessionChat";
import { SessionMessageSubmitButton } from "@/features/session-chat/SessionMessageSubmitButton";
import { useSessionQuery } from "@/features/sessions/hooks/useSessionQuery";
import { Panel } from "./Panel";
import { WsMessageType } from "@/app/websocket/messages/wsMessage";
import { useWebsocket } from "@/app/websocket/websocketProvider";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  sessionApprovalRequestQueryKey,
  type SessionApprovalRequestState,
} from "../sessions/api/sessions";
import { ApprovalAnswer } from "@/app/websocket/events/wsEvent";

type PanelSessionChatProps = {
  sessionId: string;
};

export function PanelSessionChat({ sessionId }: PanelSessionChatProps) {
  const sessionQuery = useSessionQuery(sessionId);
  const { sendWsMessage } = useWebsocket();
  const queryClient = useQueryClient();
  const { data: approvalRequestState } = useQuery({
    queryKey: sessionApprovalRequestQueryKey(sessionId),
    queryFn: () => null,
    initialData: null,
    staleTime: Infinity,
  });

  if (sessionQuery.isPending) {
    return (
      <Panel title="Session">
        <p className="text-sm text-muted-foreground">Loading messages...</p>
      </Panel>
    );
  }

  if (sessionQuery.isError) {
    return (
      <Panel title="Session">
        <p className="text-sm text-destructive">
          {sessionQuery.error instanceof Error
            ? sessionQuery.error.message
            : "Failed to load messages."}
        </p>
      </Panel>
    );
  }

  const session = sessionQuery.data;

  if (!session) {
    return (
      <Panel title="Session">
        <p className="text-sm text-muted-foreground">Session not found.</p>
      </Panel>
    );
  }

  function onDecline() {
    sendWsMessage({
      type: WsMessageType.ApprovalResponse,
      sessionId,
      answer: ApprovalAnswer.Decline,
    });
    queryClient.setQueryData<SessionApprovalRequestState>(
      sessionApprovalRequestQueryKey(sessionId),
      "responding",
    );
  }

  function onAccept() {
    sendWsMessage({
      type: WsMessageType.ApprovalResponse,
      sessionId,
      answer: ApprovalAnswer.Accept,
    });
    queryClient.setQueryData<SessionApprovalRequestState>(
      sessionApprovalRequestQueryKey(sessionId),
      "responding",
    );
  }

  function onPromptSubmit({ prompt }: { prompt: string }) {
    sendWsMessage({
      type: WsMessageType.TurnStart,
      sessionId,
      prompt,
    });
  }

  return (
    <>
      <Panel.Header>
        <Panel.Title>{session.name}</Panel.Title>
        <Panel.Description>Chat</Panel.Description>
      </Panel.Header>
      <Panel.Body className="overflow-hidden p-0">
        <SessionChat
          className="min-h-0 flex-1 overflow-y-auto px-4 py-3"
          session={session}
        />
        <SessionMessageSubmitButton
          isApprovalRequestPending={approvalRequestState !== null}
          isApprovalResponsePending={approvalRequestState === "responding"}
          onPromptSubmit={onPromptSubmit}
          onAccept={onAccept}
          onDecline={onDecline}
        />
      </Panel.Body>
    </>
  );
}
