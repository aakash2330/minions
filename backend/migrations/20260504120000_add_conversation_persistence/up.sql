CREATE TABLE workspaces (
    id TEXT NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    root_path TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX workspaces_user_updated_idx
ON workspaces(user_id, updated_at DESC);

CREATE TABLE sessions (
    session_id TEXT NOT NULL PRIMARY KEY,
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
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMP
);

CREATE INDEX sessions_workspace_updated_idx
ON sessions(workspace_id, updated_at DESC);

CREATE INDEX sessions_workspace_status_idx
ON sessions(workspace_id, status);

CREATE TABLE session_elements (
    id TEXT NOT NULL PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(session_id) ON DELETE CASCADE,
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

CREATE INDEX session_elements_session_kind_idx
ON session_elements(session_id, kind);

CREATE TABLE threads (
    id TEXT NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    session_id TEXT REFERENCES sessions(session_id) ON DELETE SET NULL,
    title TEXT NOT NULL DEFAULT 'New thread',
    codex_thread_id TEXT UNIQUE,
    cwd TEXT,
    status TEXT NOT NULL DEFAULT 'idle' CHECK (
        status IN (
            'idle',
            'starting',
            'running',
            'waiting_for_approval',
            'error'
        )
    ),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at TIMESTAMP
);

CREATE INDEX threads_user_workspace_idx
ON threads(user_id, workspace_id, updated_at DESC);

CREATE INDEX threads_session_idx
ON threads(session_id, updated_at DESC);

CREATE INDEX threads_codex_thread_idx
ON threads(codex_thread_id);

CREATE TABLE messages (
    id TEXT NOT NULL PRIMARY KEY,
    thread_id TEXT NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
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
    codex_turn_id TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE INDEX messages_thread_created_idx
ON messages(thread_id, created_at, id);

CREATE INDEX messages_codex_turn_idx
ON messages(codex_turn_id);
