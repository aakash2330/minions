import type { ComponentProps } from "react";

import {
  SessionMessageRole,
  type Session,
  type SessionMessage,
} from "@/features/sessions/api/sessions";
import { cn } from "@/lib/utils";

enum ChatSpeaker {
  Session = "session",
  Player = "player",
}

type SessionChatProps = ComponentProps<"div"> & {
  session: Session;
};

type RenderedChatMessage = {
  id: string;
  speaker: ChatSpeaker;
  text: string;
};

export function SessionChat({
  session,
  className,
  ...props
}: SessionChatProps) {
  const messages = session.messages.map(toRenderedSessionMessage);

  return (
    <div
      className={cn("flex flex-col gap-3", className)}
      {...props}
    >
      {messages.length === 0 && (
        <p className="text-sm text-muted-foreground">No messages yet.</p>
      )}
      {messages.map((message) => (
        <div
          className={
            message.speaker === ChatSpeaker.Player
              ? "ml-8 flex justify-end"
              : "mr-8 flex justify-start"
          }
          key={message.id}
        >
          <p
            className={
              message.speaker === ChatSpeaker.Player
                ? "rounded-lg bg-primary px-3 py-2 text-sm leading-5 text-primary-foreground"
                : "rounded-lg border border-border bg-muted px-3 py-2 text-sm leading-5 text-foreground"
            }
          >
            {message.text}
          </p>
        </div>
      ))}
    </div>
  );
}

function toRenderedSessionMessage(message: SessionMessage): RenderedChatMessage {
  return {
    id: message.id,
    speaker:
      message.role === SessionMessageRole.User
        ? ChatSpeaker.Player
        : ChatSpeaker.Session,
    text: message.text,
  };
}
