import { useQuery, useQueryClient } from "@tanstack/react-query";

import { ApprovalAnswer } from "@/app/websocket/events/wsEvent";
import { WsMessageType } from "@/app/websocket/messages/wsMessage";
import { useWebsocket } from "@/app/websocket/websocketProvider";
import {
  sessionApprovalRequestQueryKey,
  type SessionApprovalRequestState,
} from "@/features/sessions/api/sessions";
import { useSessionsQuery } from "@/features/sessions/hooks/useSessionsQuery";
import { GlobalChat } from "@/features/workspace-chat/GlobalChat";
import {
  WorkspaceChatMessageRole,
  workspaceChatApprovalRequestQueryKey,
  type WorkspaceChatApprovalRequestState,
  type WorkspaceChatMessage,
} from "@/features/workspace-chat/api/workspaceChat";
import { useWorkspaceChatMessagesQuery } from "@/features/workspace-chat/hooks/useWorkspaceChatMessagesQuery";

import { Panel } from "./Panel";

type PanelGlobalChatProps = {
  workspaceId: string;
};

export function PanelGlobalChat({ workspaceId }: PanelGlobalChatProps) {
  const messagesQuery = useWorkspaceChatMessagesQuery(workspaceId);
  const sessionsQuery = useSessionsQuery(workspaceId);
  const { sendWsMessage } = useWebsocket();
  const queryClient = useQueryClient();
  const { data: approvalRequestState } = useQuery({
    queryKey: workspaceChatApprovalRequestQueryKey(workspaceId),
    queryFn: (): WorkspaceChatApprovalRequestState => null,
    initialData: null,
    staleTime: Infinity,
  });
  const sessionNameById = new Map(
    (sessionsQuery.data ?? []).map((session) => [
      session.sessionId,
      session.name,
    ]),
  );

  function onDecline() {
    if (!approvalRequestState) return;

    sendWsMessage({
      type: WsMessageType.ApprovalResponse,
      sessionId: approvalRequestState.sessionId,
      answer: ApprovalAnswer.Decline,
    });
    setApprovalResponding(
      queryClient,
      workspaceId,
      approvalRequestState.sessionId,
    );
  }

  function onAccept() {
    if (!approvalRequestState) return;

    sendWsMessage({
      type: WsMessageType.ApprovalResponse,
      sessionId: approvalRequestState.sessionId,
      answer: ApprovalAnswer.Accept,
    });
    setApprovalResponding(
      queryClient,
      workspaceId,
      approvalRequestState.sessionId,
    );
  }

  function onPromptSubmit({ prompt }: { prompt: string }) {
    sendWsMessage({
      type: WsMessageType.WorkspaceChatTurnStart,
      workspaceId,
      prompt,
    });
  }

  if (messagesQuery.isPending) {
    return (
      <Panel title="Workspace Chat">
        <p className="text-sm text-muted-foreground">Loading messages...</p>
      </Panel>
    );
  }

  if (messagesQuery.isError) {
    return (
      <Panel title="Workspace Chat">
        <p className="text-sm text-destructive">
          {messagesQuery.error instanceof Error
            ? messagesQuery.error.message
            : "Failed to load messages."}
        </p>
      </Panel>
    );
  }

  return (
    <>
      <Panel.Header>
        <Panel.Title>Workspace Chat</Panel.Title>
        <Panel.Description>Global</Panel.Description>
      </Panel.Header>
      <Panel.Body className="overflow-hidden p-0">
        <GlobalChat
          messages={messagesQuery.data.map((message) =>
            toRenderedGlobalChatMessage(message, sessionNameById),
          )}
          isApprovalRequestPending={approvalRequestState !== null}
          isApprovalResponsePending={approvalRequestState?.status === "responding"}
          onPromptSubmit={onPromptSubmit}
          onAccept={onAccept}
          onDecline={onDecline}
        />
      </Panel.Body>
    </>
  );
}

function setApprovalResponding(
  queryClient: ReturnType<typeof useQueryClient>,
  workspaceId: string,
  sessionId: string,
) {
  queryClient.setQueryData<WorkspaceChatApprovalRequestState>(
    workspaceChatApprovalRequestQueryKey(workspaceId),
    { sessionId, status: "responding" },
  );
  queryClient.setQueryData<SessionApprovalRequestState>(
    sessionApprovalRequestQueryKey(sessionId),
    "responding",
  );
}

function toRenderedGlobalChatMessage(
  message: WorkspaceChatMessage,
  sessionNameById: Map<string, string>,
) {
  return {
    ...message,
    speaker: speakerForMessage(message, sessionNameById),
  };
}

function speakerForMessage(
  message: WorkspaceChatMessage,
  sessionNameById: Map<string, string>,
) {
  switch (message.role) {
    case WorkspaceChatMessageRole.User:
      return "You";
    case WorkspaceChatMessageRole.System:
      return "System";
    case WorkspaceChatMessageRole.Assistant:
      return message.sessionId
        ? (sessionNameById.get(message.sessionId) ?? message.sessionId)
        : "Assistant";
  }
}
