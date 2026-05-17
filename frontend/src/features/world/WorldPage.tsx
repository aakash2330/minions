import { useParams } from "react-router-dom";

import type { Session } from "@/features/sessions/api/sessions";
import { useSessionsQuery } from "@/features/sessions/hooks/useSessionsQuery";
import { PhaserWorld } from "@/features/world/PhaserWorld";

const EMPTY_SESSIONS: Session[] = [];

export function WorldPage() {
  const { workspaceId } = useParams();
  const sessionsQuery = useSessionsQuery(workspaceId);
  const sessions = sessionsQuery.data ?? EMPTY_SESSIONS;

  return (
    <>
      <PhaserWorld sessions={sessions} />
      {sessionsQuery.isPending && (
        <p className="mt-3 text-sm text-muted-foreground">Loading sessions...</p>
      )}
      {sessionsQuery.isError && (
        <p className="mt-3 text-sm text-destructive">
          {sessionsQuery.error instanceof Error
            ? sessionsQuery.error.message
            : "Failed to load sessions."}
        </p>
      )}
    </>
  );
}
