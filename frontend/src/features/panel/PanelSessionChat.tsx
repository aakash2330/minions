import { SessionChat } from "@/features/session-chat/SessionChat";
import { SessionMessageSubmitButton } from "@/features/session-chat/SessionMessageSubmitButton";
import { useSessionQuery } from "@/features/sessions/hooks/useSessionQuery";
import { Panel } from "./Panel";

type PanelSessionChatProps = {
  sessionId: string;
};

export function PanelSessionChat({ sessionId }: PanelSessionChatProps) {
  const sessionQuery = useSessionQuery(sessionId);

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
        <SessionMessageSubmitButton sessionId={session.sessionId} />
      </Panel.Body>
    </>
  );
}
