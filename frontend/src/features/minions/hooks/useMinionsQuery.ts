import { useQuery } from "@tanstack/react-query";

import { fetchMinions, minionsQueryKey } from "@/features/minions/api/minions";

export function useMinionsQuery() {
  return useQuery({
    queryKey: minionsQueryKey(),
    queryFn: () => fetchMinions(),
  });
}
