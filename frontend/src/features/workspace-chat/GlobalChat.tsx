import SyncLoader from "react-spinners/SyncLoader";

import {
  SessionMessageSubmitButton,
  type SessionMentionOption,
} from "@/features/session-chat/SessionMessageSubmitButton";
import {
  WorkspaceChatMessageRole,
  WorkspaceChatMessageStatus,
  type WorkspaceChatMessage,
} from "@/features/workspace-chat/api/workspaceChat";
import { cn } from "@/lib/utils";

type RenderedGlobalChatMessage = WorkspaceChatMessage & {
  speaker: string;
};

type GlobalChatProps = {
  messages: RenderedGlobalChatMessage[];
  isApprovalRequestPending: boolean;
  isApprovalResponsePending: boolean;
  mentionOptions: SessionMentionOption[];
  onAccept: () => void;
  onDecline: () => void;
  onPromptSubmit: ({ prompt }: { prompt: string }) => void;
};

export function GlobalChat({
  messages,
  isApprovalRequestPending,
  isApprovalResponsePending,
  mentionOptions,
  onPromptSubmit,
  onAccept,
  onDecline,
}: GlobalChatProps) {
  return (
    <>
      <div className="min-h-0 flex-1 space-y-3 overflow-y-auto px-4 py-3">
        {messages.length === 0 && (
          <p className="text-sm text-muted-foreground">No messages yet.</p>
        )}
        {messages.map((message) => (
          <div
            className={
              message.role === WorkspaceChatMessageRole.User
                ? "ml-8 flex justify-end"
                : "mr-8 flex justify-start"
            }
            key={message.id}
          >
            <div
              className={cn(
                "max-w-full rounded-lg px-3 py-2 text-sm leading-5",
                message.role === WorkspaceChatMessageRole.User
                  ? "bg-primary text-primary-foreground"
                  : "border border-border bg-muted text-foreground",
                message.role === WorkspaceChatMessageRole.System &&
                  "border-dashed bg-background text-muted-foreground",
              )}
            >
              <p className="mb-1 text-[11px] font-medium uppercase text-muted-foreground">
                {message.speaker}
              </p>
              {message.status === WorkspaceChatMessageStatus.Pending ? (
                <SyncLoader color="currentColor" size={5} />
              ) : (
                <p className="whitespace-pre-wrap break-words">{message.text}</p>
              )}
            </div>
          </div>
        ))}
      </div>
      <SessionMessageSubmitButton
        isApprovalRequestPending={isApprovalRequestPending}
        isApprovalResponsePending={isApprovalResponsePending}
        mentionOptions={mentionOptions}
        onPromptSubmit={onPromptSubmit}
        onAccept={onAccept}
        onDecline={onDecline}
      />
    </>
  );
}
