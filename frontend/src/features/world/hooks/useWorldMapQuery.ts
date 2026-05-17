import { useQuery } from "@tanstack/react-query";

import {
  fetchWorldMapConfig,
  worldMapQueryKey,
} from "@/features/world/api/worldMap";

export function useWorldMapQuery(workspaceId?: string) {
  const resolvedWorkspaceId = workspaceId ?? "default";

  return useQuery({
    queryKey: worldMapQueryKey(resolvedWorkspaceId),
    queryFn: () => fetchWorldMapConfig(resolvedWorkspaceId),
  });
}
