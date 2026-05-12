import { SessionChat } from "@/features/session-chat/SessionChat";
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
    <Panel title={session.name} description="Chat">
      <SessionChat session={session} />
    </Panel>
  );
}
