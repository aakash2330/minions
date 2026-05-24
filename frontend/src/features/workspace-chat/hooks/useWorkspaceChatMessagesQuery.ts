import { useQuery } from "@tanstack/react-query";

import {
  fetchWorkspaceChatMessages,
  workspaceChatMessagesQueryKey,
} from "@/features/workspace-chat/api/workspaceChat";

export function useWorkspaceChatMessagesQuery(workspaceId: string) {
  return useQuery({
    queryKey: workspaceChatMessagesQueryKey(workspaceId),
    queryFn: () => fetchWorkspaceChatMessages(workspaceId),
  });
}
