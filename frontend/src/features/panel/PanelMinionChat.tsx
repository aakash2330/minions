import { MinionChat } from "@/features/minion-chat/MinionChat";
import { useMinionQuery } from "@/features/minions/hooks/useMinionQuery";
import { Panel } from "./Panel";

type PanelMinionChatProps = {
  minionId: string;
};

export function PanelMinionChat({ minionId }: PanelMinionChatProps) {
  const minionQuery = useMinionQuery(minionId);

  if (minionQuery.isPending) {
    return (
      <Panel title="Minion">
        <p className="text-sm text-muted-foreground">Loading messages...</p>
      </Panel>
    );
  }

  if (minionQuery.isError) {
    return (
      <Panel title="Minion">
        <p className="text-sm text-destructive">
          {minionQuery.error instanceof Error
            ? minionQuery.error.message
            : "Failed to load messages."}
        </p>
      </Panel>
    );
  }

  const minion = minionQuery.data;

  if (!minion) {
    return (
      <Panel title="Minion">
        <p className="text-sm text-muted-foreground">Minion not found.</p>
      </Panel>
    );
  }

  return (
    <Panel title={minion.name} description="Chat">
      <MinionChat minion={minion} />
    </Panel>
  );
}
