use diesel::{
    dsl::insert_into,
    prelude::*,
    sql_query,
    sql_types::{Integer, Text},
    sqlite::SqliteConnection,
};
use std::{env, error::Error, path::PathBuf};

#[allow(dead_code)]
#[path = "../db.rs"]
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
    elements: &'static [SessionElementSeed],
}

struct SessionElementSeed {
    id: &'static str,
    kind: &'static str,
    label: &'static str,
    position_x: i32,
    position_y: i32,
    facing: &'static str,
}

struct ThreadSeed {
    id: &'static str,
    session_id: &'static str,
    title: &'static str,
    messages: &'static [MessageSeed],
}

struct MessageSeed {
    id: &'static str,
    role: &'static str,
    text: &'static str,
}

const KEVIN_ELEMENTS: &[SessionElementSeed] = &[SessionElementSeed {
    id: "kevin-workdesk",
    kind: "workdesk",
    label: "desk",
    position_x: 206,
    position_y: 88,
    facing: "up",
}];

const BOB_ELEMENTS: &[SessionElementSeed] = &[SessionElementSeed {
    id: "bob-workdesk",
    kind: "workdesk",
    label: "desk",
    position_x: 674,
    position_y: 88,
    facing: "up",
}];

const SESSIONS: &[SessionSeed] = &[
    SessionSeed {
        session_id: "kevin",
        name: "Kevin",
        kind: "coder",
        spawn_x: 234,
        spawn_y: 330,
        spawn_facing: "down",
        elements: KEVIN_ELEMENTS,
    },
    SessionSeed {
        session_id: "bob",
        name: "Bob",
        kind: "researcher",
        spawn_x: 702,
        spawn_y: 330,
        spawn_facing: "down",
        elements: BOB_ELEMENTS,
    },
];

const KEVIN_THREAD_MESSAGES: &[MessageSeed] = &[
    MessageSeed {
        id: "msg-kevin-setup-1",
        role: "user",
        text: "Can you inspect the backend setup and keep an eye on the Codex app-server bridge?",
    },
    MessageSeed {
        id: "msg-kevin-setup-2",
        role: "assistant",
        text: "I can handle backend implementation work, wire endpoints, and keep the app-server session flow consistent.",
    },
    MessageSeed {
        id: "msg-kevin-setup-3",
        role: "user",
        text: "Start by making sure the local database has enough data for the game UI.",
    },
];

const BOB_THREAD_MESSAGES: &[MessageSeed] = &[
    MessageSeed {
        id: "msg-bob-research-1",
        role: "user",
        text: "Track what OpenCode-style persistence needs, but keep this app simple for now.",
    },
    MessageSeed {
        id: "msg-bob-research-2",
        role: "assistant",
        text: "The current local model is workspaces, sessions, session elements, threads, and plain text messages.",
    },
];

const THREADS: &[ThreadSeed] = &[
    ThreadSeed {
        id: "conv-kevin-setup",
        session_id: "kevin",
        title: "Backend setup",
        messages: KEVIN_THREAD_MESSAGES,
    },
    ThreadSeed {
        id: "conv-bob-research",
        session_id: "bob",
        title: "Persistence notes",
        messages: BOB_THREAD_MESSAGES,
    },
];

fn main() -> Result<(), Box<dyn Error>> {
    let database_url = db::database_url();
    let mut connection = db::establish_connection(&database_url)?;

    let admin_email = env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@minions.local".to_owned());
    let admin_display_name = env::var("ADMIN_DISPLAY_NAME").unwrap_or_else(|_| "Admin".to_owned());

    insert_into(schema::users::table)
        .values((
            schema::users::email.eq(admin_email.as_str()),
            schema::users::display_name.eq(Some(admin_display_name.as_str())),
        ))
        .on_conflict(schema::users::email)
        .do_update()
        .set(schema::users::display_name.eq(Some(admin_display_name.as_str())))
        .execute(&mut connection)?;

    let admin = schema::users::table
        .select(schema::users::id)
        .filter(schema::users::email.eq(admin_email.as_str()))
        .first::<i32>(&mut connection)?;

    let workspace_id = env::var("SEED_WORKSPACE_ID").unwrap_or_else(|_| "default".to_owned());
    let workspace_name =
        env::var("SEED_WORKSPACE_NAME").unwrap_or_else(|_| "Minions Workshop".to_owned());
    let workspace_root = env::var("SEED_WORKSPACE_ROOT").unwrap_or_else(|_| {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .into_owned()
    });

    seed_workspace(
        &mut connection,
        workspace_id.as_str(),
        admin,
        workspace_name.as_str(),
        workspace_root.as_str(),
    )?;

    for session in SESSIONS {
        seed_session(&mut connection, workspace_id.as_str(), session)?;

        for element in session.elements {
            seed_session_element(&mut connection, session.session_id, element)?;
        }
    }

    for thread in THREADS {
        seed_thread(&mut connection, workspace_id.as_str(), admin, thread)?;

        for message in thread.messages {
            seed_message(&mut connection, thread.id, message)?;
        }
    }

    println!("Seeded admin user: {admin_email}");
    println!("Seeded workspace: {workspace_id}");
    println!("Seeded sessions: kevin, bob");
    println!("Seeded threads: conv-kevin-setup, conv-bob-research");
    Ok(())
}

fn seed_workspace(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    user_id: i32,
    name: &str,
    root_path: &str,
) -> QueryResult<usize> {
    sql_query(
        "
        INSERT INTO workspaces (id, user_id, name, root_path)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(id) DO UPDATE SET
            user_id = excluded.user_id,
            name = excluded.name,
            root_path = excluded.root_path,
            updated_at = CURRENT_TIMESTAMP
        ",
    )
    .bind::<Text, _>(workspace_id)
    .bind::<Integer, _>(user_id)
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

fn seed_session_element(
    connection: &mut SqliteConnection,
    session_id: &str,
    element: &SessionElementSeed,
) -> QueryResult<usize> {
    sql_query(
        "
        INSERT INTO session_elements (
            id,
            session_id,
            kind,
            label,
            position_x,
            position_y,
            facing
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(id) DO UPDATE SET
            session_id = excluded.session_id,
            kind = excluded.kind,
            label = excluded.label,
            position_x = excluded.position_x,
            position_y = excluded.position_y,
            facing = excluded.facing,
            updated_at = CURRENT_TIMESTAMP
        ",
    )
    .bind::<Text, _>(element.id)
    .bind::<Text, _>(session_id)
    .bind::<Text, _>(element.kind)
    .bind::<Text, _>(element.label)
    .bind::<Integer, _>(element.position_x)
    .bind::<Integer, _>(element.position_y)
    .bind::<Text, _>(element.facing)
    .execute(connection)
}

fn seed_thread(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    user_id: i32,
    thread: &ThreadSeed,
) -> QueryResult<usize> {
    sql_query(
        "
        INSERT INTO threads (
            id,
            user_id,
            workspace_id,
            session_id,
            title,
            status
        )
        VALUES (?1, ?2, ?3, ?4, ?5, 'idle')
        ON CONFLICT(id) DO UPDATE SET
            user_id = excluded.user_id,
            workspace_id = excluded.workspace_id,
            session_id = excluded.session_id,
            title = excluded.title,
            updated_at = CURRENT_TIMESTAMP
        ",
    )
    .bind::<Text, _>(thread.id)
    .bind::<Integer, _>(user_id)
    .bind::<Text, _>(workspace_id)
    .bind::<Text, _>(thread.session_id)
    .bind::<Text, _>(thread.title)
    .execute(connection)
}

fn seed_message(
    connection: &mut SqliteConnection,
    thread_id: &str,
    message: &MessageSeed,
) -> QueryResult<usize> {
    sql_query(
        "
        INSERT INTO messages (
            id,
            thread_id,
            role,
            text,
            status,
            completed_at
        )
        VALUES (?1, ?2, ?3, ?4, 'complete', CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
            thread_id = excluded.thread_id,
            role = excluded.role,
            text = excluded.text,
            status = excluded.status,
            completed_at = excluded.completed_at,
            updated_at = CURRENT_TIMESTAMP
        ",
    )
    .bind::<Text, _>(message.id)
    .bind::<Text, _>(thread_id)
    .bind::<Text, _>(message.role)
    .bind::<Text, _>(message.text)
    .execute(connection)
}
