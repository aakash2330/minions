import { useQuery } from "@tanstack/react-query";

import {
  fetchWorkspaces,
  workspacesQueryKey,
} from "@/features/workspaces/api/workspaces";

export function useWorkspacesQuery() {
  return useQuery({
    queryKey: workspacesQueryKey(),
    queryFn: fetchWorkspaces,
  });
}
