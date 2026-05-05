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

CREATE TABLE minions (
    id TEXT NOT NULL PRIMARY KEY,
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

CREATE INDEX minions_workspace_updated_idx
ON minions(workspace_id, updated_at DESC);

CREATE INDEX minions_workspace_status_idx
ON minions(workspace_id, status);

CREATE TABLE minion_elements (
    id TEXT NOT NULL PRIMARY KEY,
    minion_id TEXT NOT NULL REFERENCES minions(id) ON DELETE CASCADE,
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

CREATE INDEX minion_elements_minion_kind_idx
ON minion_elements(minion_id, kind);

CREATE TABLE conversations (
    id TEXT NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    minion_id TEXT REFERENCES minions(id) ON DELETE SET NULL,
    title TEXT NOT NULL DEFAULT 'New conversation',
    codex_thread_id TEXT UNIQUE,
    current_session_id TEXT,
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

CREATE INDEX conversations_user_workspace_idx
ON conversations(user_id, workspace_id, updated_at DESC);

CREATE INDEX conversations_minion_idx
ON conversations(minion_id, updated_at DESC);

CREATE INDEX conversations_codex_thread_idx
ON conversations(codex_thread_id);

CREATE TABLE messages (
    id TEXT NOT NULL PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
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

CREATE INDEX messages_conversation_created_idx
ON messages(conversation_id, created_at, id);

CREATE INDEX messages_codex_turn_idx
ON messages(codex_turn_id);
