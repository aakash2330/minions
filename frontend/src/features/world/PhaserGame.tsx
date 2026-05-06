import { type FormEvent, useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { SendHorizontal } from "lucide-react";
import { AUTO, Game } from "phaser";

import {
  ConversationMessageRole,
  fetchHistoricalConversations,
  type HistoricalConversation,
  type HistoricalConversationMessage,
} from "@/features/conversations/api/conversations";
import {
  fetchMinions,
  getMinionConfigBySessionId,
} from "@/features/minions/api/minions";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Sheet,
  SheetContent,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import {
  ChatSpeaker,
  type ChatMessage,
  useMinionChatStore,
} from "@/features/minion-chat/stores/minionChatStore";
import type { MinionMapConfig } from "@/game/minionMapConfig";
import { canUseGameKeyboardInput } from "@/game/input/keyboardControlGate";
import { WorldScene } from "@/game/WorldScene";

const EMPTY_MESSAGES: ChatMessage[] = [];
const EMPTY_CONVERSATIONS: HistoricalConversation[] = [];
const EMPTY_HISTORICAL_MESSAGES: HistoricalConversationMessage[] = [];
const EMPTY_MINIONS: MinionMapConfig[] = [];

type RenderedChatMessage = {
  id: string;
  speaker: ChatSpeaker;
  text: string;
};

const EMPTY_RENDERED_CHAT_MESSAGES: RenderedChatMessage[] = [];

export function PhaserGame() {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const gameRef = useRef<Game | null>(null);
  const minionsQuery = useQuery({
    queryKey: ["minions"],
    queryFn: fetchMinions,
  });
  const conversationsQuery = useQuery({
    queryKey: ["historical-conversations"],
    queryFn: fetchHistoricalConversations,
  });
  const minions = minionsQuery.data ?? EMPTY_MINIONS;
  const conversations = conversationsQuery.data ?? EMPTY_CONVERSATIONS;
  const draftMessagesBySessionId = useMinionChatStore(
    (state) => state.draftMessagesBySessionId,
  );
  const messagesBySessionId = useMinionChatStore(
    (state) => state.messagesBySessionId,
  );
  const minionChatOpen = useMinionChatStore((state) => state.isOpen);
  const selectedSessionId = useMinionChatStore(
    (state) => state.selectedSessionId,
  );
  const sendPlayerMessage = useMinionChatStore(
    (state) => state.sendPlayerMessage,
  );
  const setDraftMessage = useMinionChatStore(
    (state) => state.setDraftMessage,
  );
  const setMinionChatOpen = useMinionChatStore((state) => state.setOpen);
  const selectedMinionConfig = selectedSessionId
    ? getMinionConfigBySessionId(minions, selectedSessionId)
    : undefined;
  const historicalMessages = selectedSessionId
    ? getLatestSessionConversationMessages(conversations, selectedSessionId).map(
        toRenderedHistoricalMessage,
      )
    : EMPTY_RENDERED_CHAT_MESSAGES;
  const localMessages = selectedSessionId
    ? (messagesBySessionId[selectedSessionId] ?? EMPTY_MESSAGES)
    : EMPTY_MESSAGES;
  const messages: RenderedChatMessage[] = [
    ...historicalMessages,
    ...localMessages.map(toRenderedLocalMessage),
  ];
  const draftMessage = selectedSessionId
    ? (draftMessagesBySessionId[selectedSessionId] ?? "")
    : "";

  useEffect(() => {
    if (!parentRef.current || gameRef.current || !minionsQuery.isSuccess) {
      return;
    }

    gameRef.current = new Game({
      type: AUTO,
      parent: parentRef.current,
      backgroundColor: "#161a1d",
      width: 960,
      height: 540,
      render: {
        pixelArt: true,
      },
      scene: [
        new WorldScene({
          canUseKeyboardInput: () =>
            canUseGameKeyboardInput({
              disabled: useMinionChatStore.getState().isOpen,
            }),
          minions,
          onMinionChat: (config) => {
            useMinionChatStore.getState().openChat(config.sessionId);
          },
        }),
      ],
    });

    return () => {
      gameRef.current?.destroy(true);
      gameRef.current = null;
    };
  }, [minionsQuery.isSuccess, minions]);

  function handleSendMessage(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    sendPlayerMessage();
  }

  return (
    <>
      <div className="phaser-game" ref={parentRef} />
      {minionsQuery.isPending && (
        <p className="mt-3 text-sm text-muted-foreground">Loading map...</p>
      )}
      {minionsQuery.isError && (
        <p className="mt-3 text-sm text-destructive">
          {minionsQuery.error instanceof Error
            ? minionsQuery.error.message
            : "Failed to load map."}
        </p>
      )}
      <Sheet open={minionChatOpen} onOpenChange={setMinionChatOpen}>
        <SheetContent className="border-border/80 bg-popover p-0 text-popover-foreground">
          <SheetHeader className="border-b border-border px-4 py-3">
            <SheetTitle>{selectedMinionConfig?.name ?? "minion"}</SheetTitle>
          </SheetHeader>

          <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto px-4 py-3">
            {selectedSessionId && conversationsQuery.isPending && (
              <p className="text-sm text-muted-foreground">Loading chat...</p>
            )}
            {selectedSessionId && conversationsQuery.isError && (
              <p className="text-sm text-destructive">
                {conversationsQuery.error instanceof Error
                  ? conversationsQuery.error.message
                  : "Failed to load chat history."}
              </p>
            )}
            {selectedSessionId &&
              conversationsQuery.isSuccess &&
              messages.length === 0 && (
                <p className="text-sm text-muted-foreground">
                  No messages yet.
                </p>
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

          <SheetFooter className="border-t border-border p-3">
            <form className="flex w-full gap-2" onSubmit={handleSendMessage}>
              <Input
                aria-label="Message minion"
                autoComplete="off"
                autoFocus
                className="h-9 flex-1"
                onChange={(event) => {
                  setDraftMessage(event.target.value);
                }}
                placeholder="Message minion"
                value={draftMessage}
              />
              <Button
                aria-label="Send message"
                size="icon-lg"
                className="shrink-0"
                type="submit"
              >
                <SendHorizontal />
              </Button>
            </form>
          </SheetFooter>
        </SheetContent>
      </Sheet>
    </>
  );
}

function getLatestSessionConversationMessages(
  conversations: HistoricalConversation[],
  sessionId: string,
) {
  return (
    conversations.find((conversation) => conversation.sessionId === sessionId)
      ?.messages ?? EMPTY_HISTORICAL_MESSAGES
  );
}

function toRenderedHistoricalMessage(
  message: HistoricalConversationMessage,
): RenderedChatMessage {
  return {
    id: `history-${message.id}`,
    speaker:
      message.role === ConversationMessageRole.User
        ? ChatSpeaker.Player
        : ChatSpeaker.Minion,
    text: message.text,
  };
}

function toRenderedLocalMessage(message: ChatMessage): RenderedChatMessage {
  return {
    id: `local-${message.id}`,
    speaker: message.speaker,
    text: message.text,
  };
}
