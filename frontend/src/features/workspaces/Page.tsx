import { Link } from "react-router-dom";

import { useWorkspacesQuery } from "@/features/workspaces/hooks/useWorkspacesQuery";

export function WorkspacesPage() {
  const workspacesQuery = useWorkspacesQuery();

  if (workspacesQuery.isPending) {
    return <p className="text-sm text-muted-foreground">Loading workspaces...</p>;
  }

  if (workspacesQuery.isError) {
    return (
      <p className="text-sm text-destructive">
        {workspacesQuery.error instanceof Error
          ? workspacesQuery.error.message
          : "Failed to load workspaces."}
      </p>
    );
  }

  return (
    <section className="w-full max-w-5xl space-y-4">
      <h1 className="text-xl font-semibold">Workspaces</h1>
      {workspacesQuery.data.length === 0 ? (
        <p className="text-sm text-muted-foreground">No workspaces yet.</p>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {workspacesQuery.data.map((workspace) => (
            <Link
              className="block rounded-lg border border-border bg-card p-4 text-card-foreground transition hover:border-primary/60 hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              key={workspace.id}
              to={`/workspace/${encodeURIComponent(workspace.id)}/world`}
            >
              <h2 className="text-sm font-medium">{workspace.name}</h2>
              {workspace.rootPath && (
                <p className="mt-2 truncate text-xs text-muted-foreground">
                  {workspace.rootPath}
                </p>
              )}
            </Link>
          ))}
        </div>
      )}
    </section>
  );
}
