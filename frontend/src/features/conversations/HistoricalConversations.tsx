import { useQuery } from "@tanstack/react-query";
import { MessageSquareText, RefreshCw } from "lucide-react";

import {
  fetchHistoricalConversations,
  historicalConversationsQueryKey,
  type HistoricalConversation,
} from "@/features/conversations/api/conversations";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";

const EMPTY_CONVERSATIONS: HistoricalConversation[] = [];

export function HistoricalConversations() {
  const conversationsQuery = useQuery({
    queryKey: historicalConversationsQueryKey,
    queryFn: fetchHistoricalConversations,
  });
  const conversations = conversationsQuery.data ?? EMPTY_CONVERSATIONS;

  return (
    <section className="mx-auto flex min-h-[calc(100vh-48px)] w-full max-w-5xl flex-col gap-5">
      <header className="flex items-center justify-between gap-4">
        <div className="min-w-0">
          <h1 className="text-xl font-medium tracking-normal text-foreground">
            Conversations
          </h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Historical conversations from the workspace.
          </p>
        </div>
        <Button
          disabled={conversationsQuery.isFetching}
          onClick={() => {
            void conversationsQuery.refetch();
          }}
          variant="outline"
        >
          <RefreshCw />
          Refresh
        </Button>
      </header>

      <Separator />

      {conversationsQuery.isPending && (
        <div className="rounded-lg border border-border bg-card p-4 text-sm text-muted-foreground">
          Loading conversations...
        </div>
      )}

      {conversationsQuery.isError && (
        <div className="rounded-lg border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive">
          {conversationsQuery.error instanceof Error
            ? conversationsQuery.error.message
            : "Failed to load conversations."}
        </div>
      )}

      {conversationsQuery.isSuccess && conversations.length === 0 && (
        <div className="grid min-h-64 place-items-center rounded-lg border border-dashed border-border bg-card/40 p-8 text-center">
          <div>
            <MessageSquareText className="mx-auto size-8 text-muted-foreground" />
            <p className="mt-3 text-sm font-medium text-foreground">
              No conversations yet
            </p>
            <p className="mt-1 text-sm text-muted-foreground">
              Historical conversations will appear here once they are persisted.
            </p>
          </div>
        </div>
      )}

      {conversations.length > 0 && (
        <div className="grid gap-3">
          {conversations.map((conversation) => {
            const lastMessage =
              conversation.messages[conversation.messages.length - 1];

            return (
              <article
                className="rounded-lg border border-border bg-card p-4 text-card-foreground"
                key={conversation.id}
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="min-w-0">
                    <h2 className="truncate text-sm font-medium">
                      {conversation.title}
                    </h2>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {conversation.minionName ?? "Unassigned"} ·{" "}
                      {conversation.status} · {conversation.messages.length}{" "}
                      messages
                    </p>
                  </div>
                  <p className="shrink-0 text-xs text-muted-foreground">
                    {conversation.workspaceId}
                  </p>
                </div>

                {lastMessage && (
                  <p className="mt-3 line-clamp-2 text-sm leading-5 text-muted-foreground">
                    <span className="text-foreground">{lastMessage.role}:</span>{" "}
                    {lastMessage.text}
                  </p>
                )}

                {conversation.workspaceRootPath && (
                  <p className="mt-3 truncate border-t border-border pt-3 text-xs text-muted-foreground">
                    {conversation.workspaceRootPath}
                  </p>
                )}
              </article>
            );
          })}
        </div>
      )}
    </section>
  );
}
