import { useQuery } from "@tanstack/react-query";

import { fetchMinion, minionQueryKey } from "@/features/minions/api/minions";

export function useMinionQuery(minionId: string) {
  return useQuery({
    queryKey: minionQueryKey(minionId),
    queryFn: () => fetchMinion(minionId),
  });
}
