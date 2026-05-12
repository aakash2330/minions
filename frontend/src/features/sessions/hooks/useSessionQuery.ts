import { useQuery } from "@tanstack/react-query";

import { fetchSession, sessionQueryKey } from "@/features/sessions/api/sessions";

export function useSessionQuery(sessionId: string) {
  return useQuery({
    queryKey: sessionQueryKey(sessionId),
    queryFn: () => fetchSession(sessionId),
  });
}
