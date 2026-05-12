import type { ComponentProps } from "react";

import {
  MinionMessageRole,
  type Minion,
  type MinionMessage,
} from "@/features/minions/api/minions";
import { cn } from "@/lib/utils";

enum ChatSpeaker {
  Minion = "minion",
  Player = "player",
}

type MinionChatProps = ComponentProps<"div"> & {
  minion: Minion;
};

type RenderedChatMessage = {
  id: string;
  speaker: ChatSpeaker;
  text: string;
};

export function MinionChat({
  minion,
  className,
  ...props
}: MinionChatProps) {
  const messages = minion.messages.map(toRenderedMinionMessage);

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

function toRenderedMinionMessage(message: MinionMessage): RenderedChatMessage {
  return {
    id: message.id,
    speaker:
      message.role === MinionMessageRole.User
        ? ChatSpeaker.Player
        : ChatSpeaker.Minion,
    text: message.text,
  };
}
