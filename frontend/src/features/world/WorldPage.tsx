import type { Minion } from "@/features/minions/api/minions";
import { useMinionsQuery } from "@/features/minions/hooks/useMinionsQuery";
import { PhaserWorld } from "@/features/world/PhaserWorld";

const EMPTY_MINIONS: Minion[] = [];

export function WorldPage() {
  const minionsQuery = useMinionsQuery();
  const minions = minionsQuery.data ?? EMPTY_MINIONS;

  return (
    <>
      {minionsQuery.isSuccess && (
        <PhaserWorld minions={minions} />
      )}
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
    </>
  );
}
