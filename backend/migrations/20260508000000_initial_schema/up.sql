CREATE TABLE workspaces (
    id TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    root_path TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX workspaces_updated_idx
ON workspaces(updated_at DESC);

CREATE TABLE sessions (
    session_id TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (
        kind IN ('researcher', 'coder', 'reviewer', 'openclaw')
    ),
    status TEXT NOT NULL DEFAULT 'idle' CHECK (
        status IN (
            'idle',
            'moving',
            'working',
            'error',
            'archived'
        )
    ),
    spawn_x INTEGER NOT NULL,
    spawn_y INTEGER NOT NULL,
    spawn_facing TEXT NOT NULL DEFAULT 'down' CHECK (
        spawn_facing IN ('up', 'left', 'down', 'right')
    ),
    current_x INTEGER NOT NULL,
    current_y INTEGER NOT NULL,
    current_facing TEXT NOT NULL DEFAULT 'down' CHECK (
        current_facing IN ('up', 'left', 'down', 'right')
    ),
    codex_thread_id TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMP
);

CREATE INDEX sessions_workspace_updated_idx
ON sessions(workspace_id, updated_at DESC);

CREATE INDEX sessions_workspace_status_idx
ON sessions(workspace_id, status);

CREATE UNIQUE INDEX sessions_codex_thread_idx
ON sessions(codex_thread_id)
WHERE codex_thread_id IS NOT NULL;

CREATE TABLE workspace_elements (
    id TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    assigned_session_id TEXT REFERENCES sessions(session_id) ON DELETE SET NULL,
    kind TEXT NOT NULL,
    label TEXT NOT NULL,
    position_x INTEGER NOT NULL,
    position_y INTEGER NOT NULL,
    facing TEXT NOT NULL DEFAULT 'up' CHECK (
        facing IN ('up', 'left', 'down', 'right')
    ),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX workspace_elements_workspace_kind_idx
ON workspace_elements(workspace_id, kind);

CREATE INDEX workspace_elements_assigned_session_idx
ON workspace_elements(assigned_session_id);

CREATE TABLE messages (
    id TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    session_id TEXT NOT NULL REFERENCES sessions(session_id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    text TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'complete' CHECK (
        status IN (
            'pending',
            'streaming',
            'complete',
            'error'
        )
    ),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE INDEX messages_session_created_idx
ON messages(session_id, created_at, id);
