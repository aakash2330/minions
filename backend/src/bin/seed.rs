use diesel::{
    prelude::*,
    sql_query,
    sql_types::{Integer, Text},
    sqlite::SqliteConnection,
};
use std::{env, error::Error, path::PathBuf};

#[allow(dead_code)]
#[path = "../infrastructure/db/pool.rs"]
mod db;

#[path = "../schema.rs"]
mod schema;

struct SessionSeed {
    session_id: &'static str,
    name: &'static str,
    kind: &'static str,
    spawn_x: i32,
    spawn_y: i32,
    spawn_facing: &'static str,
    messages: &'static [MessageSeed],
}

struct WorkspaceElementSeed {
    id: &'static str,
    assigned_session_id: Option<&'static str>,
    kind: &'static str,
    label: &'static str,
    position_x: i32,
    position_y: i32,
    facing: &'static str,
}

struct MessageSeed {
    role: &'static str,
    text: &'static str,
}

const SESSIONS: &[SessionSeed] = &[
    SessionSeed {
        session_id: "kevin",
        name: "Kevin",
        kind: "coder",
        spawn_x: 234,
        spawn_y: 330,
        spawn_facing: "down",
        messages: KEVIN_THREAD_MESSAGES,
    },
    SessionSeed {
        session_id: "bob",
        name: "Bob",
        kind: "researcher",
        spawn_x: 702,
        spawn_y: 330,
        spawn_facing: "down",
        messages: BOB_THREAD_MESSAGES,
    },
];

const WORKSPACE_ELEMENTS: &[WorkspaceElementSeed] = &[
    WorkspaceElementSeed {
        id: "kevin-workdesk",
        assigned_session_id: Some("kevin"),
        kind: "workdesk",
        label: "desk",
        position_x: 206,
        position_y: 88,
        facing: "up",
    },
    WorkspaceElementSeed {
        id: "bob-workdesk",
        assigned_session_id: Some("bob"),
        kind: "workdesk",
        label: "desk",
        position_x: 674,
        position_y: 88,
        facing: "up",
    },
];

const KEVIN_THREAD_MESSAGES: &[MessageSeed] = &[
    MessageSeed {
        role: "user",
        text: "Can you inspect the backend setup and keep an eye on the Codex app-server bridge?",
    },
    MessageSeed {
        role: "assistant",
        text: "I can handle backend implementation work, wire endpoints, and keep the app-server session flow consistent.",
    },
    MessageSeed {
        role: "user",
        text: "Start by making sure the local database has enough data for the game UI.",
    },
];

const BOB_THREAD_MESSAGES: &[MessageSeed] = &[
    MessageSeed {
        role: "user",
        text: "Track what OpenCode-style persistence needs, but keep this app simple for now.",
    },
    MessageSeed {
        role: "assistant",
        text: "The current local model is workspaces, workspace elements, sessions, and plain text messages.",
    },
];

fn main() -> Result<(), Box<dyn Error>> {
    let database_url = db::database_url();
    let mut connection = db::establish_connection(&database_url)?;

    let workspace_id = env::var("SEED_WORKSPACE_ID").unwrap_or_else(|_| "default".to_owned());
    let workspace_name =
        env::var("SEED_WORKSPACE_NAME").unwrap_or_else(|_| "Sessions Workshop".to_owned());
    let workspace_root = env::var("SEED_WORKSPACE_ROOT").unwrap_or_else(|_| {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .into_owned()
    });

    seed_workspace(
        &mut connection,
        workspace_id.as_str(),
        workspace_name.as_str(),
        workspace_root.as_str(),
    )?;

    for session in SESSIONS {
        seed_session(&mut connection, workspace_id.as_str(), session)?;

        for message in session.messages {
            seed_message(&mut connection, session.session_id, message)?;
        }
    }

    for element in WORKSPACE_ELEMENTS {
        seed_workspace_element(&mut connection, workspace_id.as_str(), element)?;
    }

    println!("Seeded workspace: {workspace_id}");
    println!("Seeded sessions: kevin, bob");
    Ok(())
}

fn seed_workspace(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    name: &str,
    root_path: &str,
) -> QueryResult<usize> {
    sql_query(
        "
        INSERT INTO workspaces (id, name, root_path)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            root_path = excluded.root_path,
            updated_at = CURRENT_TIMESTAMP
        ",
    )
    .bind::<Text, _>(workspace_id)
    .bind::<Text, _>(name)
    .bind::<Text, _>(root_path)
    .execute(connection)
}

fn seed_session(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    session: &SessionSeed,
) -> QueryResult<usize> {
    sql_query(
        "
        INSERT INTO sessions (
            session_id,
            workspace_id,
            name,
            kind,
            spawn_x,
            spawn_y,
            spawn_facing,
            current_x,
            current_y,
            current_facing
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(session_id) DO UPDATE SET
            workspace_id = excluded.workspace_id,
            name = excluded.name,
            kind = excluded.kind,
            spawn_x = excluded.spawn_x,
            spawn_y = excluded.spawn_y,
            spawn_facing = excluded.spawn_facing,
            updated_at = CURRENT_TIMESTAMP
        ",
    )
    .bind::<Text, _>(session.session_id)
    .bind::<Text, _>(workspace_id)
    .bind::<Text, _>(session.name)
    .bind::<Text, _>(session.kind)
    .bind::<Integer, _>(session.spawn_x)
    .bind::<Integer, _>(session.spawn_y)
    .bind::<Text, _>(session.spawn_facing)
    .bind::<Integer, _>(session.spawn_x)
    .bind::<Integer, _>(session.spawn_y)
    .bind::<Text, _>(session.spawn_facing)
    .execute(connection)
}

fn seed_workspace_element(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    element: &WorkspaceElementSeed,
) -> QueryResult<usize> {
    sql_query(
        "
        INSERT INTO workspace_elements (
            id,
            workspace_id,
            assigned_session_id,
            kind,
            label,
            position_x,
            position_y,
            facing
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            workspace_id = excluded.workspace_id,
            assigned_session_id = excluded.assigned_session_id,
            kind = excluded.kind,
            label = excluded.label,
            position_x = excluded.position_x,
            position_y = excluded.position_y,
            facing = excluded.facing,
            updated_at = CURRENT_TIMESTAMP
        ",
    )
    .bind::<Text, _>(element.id)
    .bind::<Text, _>(workspace_id)
    .bind::<diesel::sql_types::Nullable<Text>, _>(element.assigned_session_id)
    .bind::<Text, _>(element.kind)
    .bind::<Text, _>(element.label)
    .bind::<Integer, _>(element.position_x)
    .bind::<Integer, _>(element.position_y)
    .bind::<Text, _>(element.facing)
    .execute(connection)
}

fn seed_message(
    connection: &mut SqliteConnection,
    session_id: &str,
    message: &MessageSeed,
) -> QueryResult<usize> {
    sql_query(
        "
        INSERT INTO messages (
            session_id,
            role,
            text,
            status,
            completed_at
        )
        VALUES (?1, ?2, ?3, 'complete', CURRENT_TIMESTAMP)
        ",
    )
    .bind::<Text, _>(session_id)
    .bind::<Text, _>(message.role)
    .bind::<Text, _>(message.text)
    .execute(connection)
}
