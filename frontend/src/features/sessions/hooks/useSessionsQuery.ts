import { useQuery } from "@tanstack/react-query";

import { fetchSessions, sessionsQueryKey } from "@/features/sessions/api/sessions";

export function useSessionsQuery() {
  return useQuery({
    queryKey: sessionsQueryKey(),
    queryFn: () => fetchSessions(),
  });
}
