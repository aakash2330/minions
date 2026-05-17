import { useParams } from "react-router-dom";

import type { Session } from "@/features/sessions/api/sessions";
import { useSessionsQuery } from "@/features/sessions/hooks/useSessionsQuery";
import { PhaserWorld } from "@/features/world/PhaserWorld";
import { useWorldMapQuery } from "@/features/world/hooks/useWorldMapQuery";

const EMPTY_SESSIONS: Session[] = [];

export function WorldPage() {
  const { workspaceId } = useParams();
  const sessionsQuery = useSessionsQuery(workspaceId);
  const worldMapQuery = useWorldMapQuery(workspaceId);
  const sessions = sessionsQuery.data ?? EMPTY_SESSIONS;

  return (
    <>
      {worldMapQuery.data && (
        <PhaserWorld mapConfig={worldMapQuery.data} sessions={sessions} />
      )}
      {worldMapQuery.isPending && (
        <p className="mt-3 text-sm text-muted-foreground">Loading map...</p>
      )}
      {worldMapQuery.isError && (
        <p className="mt-3 text-sm text-destructive">
          {worldMapQuery.error instanceof Error
            ? worldMapQuery.error.message
            : "Failed to load map."}
        </p>
      )}
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
