import { useQuery } from "@tanstack/react-query";

import { fetchSessions, sessionsQueryKey } from "@/features/sessions/api/sessions";

export function useSessionsQuery(workspaceId?: string) {
  return useQuery({
    queryKey: sessionsQueryKey(workspaceId),
    queryFn: () => fetchSessions(workspaceId),
  });
}
