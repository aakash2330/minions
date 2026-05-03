import { type FormEvent, useEffect, useRef, useState } from "react";
import { SendHorizontal } from "lucide-react";
import { AUTO, Game } from "phaser";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Sheet,
  SheetContent,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { canUseGameKeyboardInput } from "./game/input/keyboardControlGate";
import { WorldScene } from "./game/WorldScene";

type ChatMessage = {
  id: number;
  speaker: "minion" | "player";
  text: string;
};

const INITIAL_MESSAGES: ChatMessage[] = [
  {
    id: 1,
    speaker: "minion",
    text: "You found me.",
  },
  {
    id: 2,
    speaker: "player",
    text: "What are you doing here?",
  },
  {
    id: 3,
    speaker: "minion",
    text: "Waiting for something to do.",
  },
];

export function PhaserGame() {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const gameRef = useRef<Game | null>(null);
  const gameKeyboardDisabledRef = useRef(false);
  const [minionChatOpen, setMinionChatOpen] = useState(false);
  const [selectedMinionName, setSelectedMinionName] = useState("minion");
  const [draftMessage, setDraftMessage] = useState("");
  const [messages, setMessages] = useState(INITIAL_MESSAGES);

  useEffect(() => {
    gameKeyboardDisabledRef.current = minionChatOpen;
  }, [minionChatOpen]);

  useEffect(() => {
    if (!parentRef.current || gameRef.current) {
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
              disabled: gameKeyboardDisabledRef.current,
            }),
          onMinionChat: (config) => {
            setSelectedMinionName(config.name);
            setMinionChatOpen(true);
          },
        }),
      ],
    });

    return () => {
      gameRef.current?.destroy(true);
      gameRef.current = null;
    };
  }, []);

  function handleSendMessage(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const text = draftMessage.trim();

    if (!text) {
      return;
    }

    setMessages((currentMessages) => [
      ...currentMessages,
      {
        id: Date.now(),
        speaker: "player",
        text,
      },
    ]);
    setDraftMessage("");
  }

  return (
    <>
      <div className="phaser-game" ref={parentRef} />
      <Sheet open={minionChatOpen} onOpenChange={setMinionChatOpen}>
        <SheetContent className="border-border/80 bg-popover p-0 text-popover-foreground">
          <SheetHeader className="border-b border-border px-4 py-3">
            <SheetTitle>{selectedMinionName}</SheetTitle>
          </SheetHeader>

          <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto px-4 py-3">
            {messages.map((message) => (
              <div
                className={
                  message.speaker === "player"
                    ? "ml-8 flex justify-end"
                    : "mr-8 flex justify-start"
                }
                key={message.id}
              >
                <p
                  className={
                    message.speaker === "player"
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
