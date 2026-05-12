import { useEffect, useRef } from "react";
import { AUTO, Game } from "phaser";

import { usePanelStore } from "@/features/panel/stores/panelStore";
import { canUseGameKeyboardInput } from "@/game/input/keyboardControlGate";
import type { SessionMapConfig } from "@/game/sessionMapConfig";
import { WorldScene } from "@/game/WorldScene";

type PhaserWorldProps = {
  sessions: SessionMapConfig[];
};

export function PhaserWorld({ sessions }: PhaserWorldProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const gameRef = useRef<Game | null>(null);

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
          canUseKeyboardInput: canUseWorldKeyboardInput,
          sessions,
        }),
      ],
    });

    return () => {
      gameRef.current?.destroy(true);
      gameRef.current = null;
    };
  }, [sessions]);

  return <div className="phaser-game" ref={parentRef} />;
}

function canUseWorldKeyboardInput() {
  return canUseGameKeyboardInput({
    disabled: usePanelStore.getState().isOpen,
  });
}
