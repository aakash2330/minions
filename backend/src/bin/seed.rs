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

#[path = "../domain/enums.rs"]
mod enums;

#[path = "../schema.rs"]
mod schema;

use enums::{Direction, WorkspaceElementKind};

struct SessionSeed {
    session_id: &'static str,
    name: &'static str,
    kind: &'static str,
    spawn_x: i32,
    spawn_y: i32,
    spawn_facing: Direction,
    messages: &'static [MessageSeed],
}

struct WorkspaceElementSeed {
    id: &'static str,
    assigned_session_id: Option<&'static str>,
    kind: WorkspaceElementKind,
    label: &'static str,
    position_x: i32,
    position_y: i32,
    facing: Direction,
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
        spawn_x: 230,
        spawn_y: 405,
        spawn_facing: Direction::UpRight,
        messages: KEVIN_THREAD_MESSAGES,
    },
    SessionSeed {
        session_id: "bob",
        name: "Bob",
        kind: "researcher",
        spawn_x: 1155,
        spawn_y: 420,
        spawn_facing: Direction::UpRight,
        messages: BOB_THREAD_MESSAGES,
    },
];

const WORKSPACE_ELEMENTS: &[WorkspaceElementSeed] = &[
    WorkspaceElementSeed {
        id: "kevin-workdesk",
        assigned_session_id: Some("kevin"),
        kind: WorkspaceElementKind::PersonalTable,
        label: "personal table",
        position_x: 230,
        position_y: 405,
        facing: Direction::UpRight,
    },
    WorkspaceElementSeed {
        id: "bob-workdesk",
        assigned_session_id: Some("bob"),
        kind: WorkspaceElementKind::PersonalTable,
        label: "personal table",
        position_x: 1155,
        position_y: 420,
        facing: Direction::UpRight,
    },
    WorkspaceElementSeed {
        id: "kevin-meeting-table",
        assigned_session_id: Some("kevin"),
        kind: WorkspaceElementKind::MeetingTable,
        label: "meeting table",
        position_x: 720,
        position_y: 645,
        facing: Direction::UpRight,
    },
    WorkspaceElementSeed {
        id: "bob-meeting-table",
        assigned_session_id: Some("bob"),
        kind: WorkspaceElementKind::MeetingTable,
        label: "meeting table",
        position_x: 900,
        position_y: 705,
        facing: Direction::UpRight,
    },
];

const WORKSPACE_MAP_CONFIG_JSON: &str = r##"
{
  "canvas": {
    "background": "#edf0df",
    "gridSize": 4,
    "height": 900,
    "width": 1600
  },
  "items": [
    {
      "assetId": "tinyhouse-living-roon-book-8-eab057af",
      "facing": "down-right",
      "flipX": false,
      "height": 66,
      "id": "starter-31-central-book-stack",
      "kind": "book-stack",
      "label": "central book stack",
      "rotation": 0,
      "width": 66,
      "x": 832,
      "y": 521
    },
    {
      "assetId": "tinyhouse-plants-cactus-1-c17c3165",
      "facing": "down-right",
      "flipX": false,
      "height": 75,
      "id": "starter-38-right-desk-cactus",
      "kind": "cactus",
      "label": "right desk cactus",
      "rotation": 0,
      "width": 75,
      "x": 1198,
      "y": 209
    },
    {
      "assetId": "tinyhouse-pc-oldpc-downleft-tile-dfbd9faf",
      "facing": "down-left",
      "flipX": false,
      "height": 144,
      "id": "49c929d1-cb2e-4a83-9cc0-f035ba6d4751",
      "kind": "computer",
      "label": "right computer",
      "rotation": 0,
      "width": 164,
      "x": 1232,
      "y": 192
    },
    {
      "assetId": "tinyhouse-desk-desk-1-downright-tile-ae7b680f",
      "facing": "down-right",
      "flipX": false,
      "height": 536,
      "id": "starter-14-left-desk",
      "kind": "desk",
      "label": "left desk",
      "rotation": 0,
      "width": 624,
      "x": 64,
      "y": 72
    },
    {
      "assetId": "tinyhouse-desk-desk-1-downright-tile-ae7b680f",
      "facing": "down-right",
      "flipX": false,
      "height": 564,
      "id": "starter-15-right-desk",
      "kind": "desk",
      "label": "right desk",
      "rotation": 0,
      "width": 568,
      "x": 1020,
      "y": 88
    },
    {
      "assetId": "tinyhouse-pc-oldkeyboard-downright-tile-d6a6d20f",
      "facing": "down-right",
      "flipX": false,
      "height": 84,
      "id": "starter-25-left-keyboard",
      "kind": "keyboard",
      "label": "left keyboard",
      "rotation": 0,
      "width": 104,
      "x": 256,
      "y": 286
    },
    {
      "assetId": "tinyhouse-pc-oldkeyboard-downright-tile-d6a6d20f",
      "facing": "down-right",
      "flipX": false,
      "height": 56,
      "id": "starter-26-right-keyboard",
      "kind": "keyboard",
      "label": "right keyboard",
      "rotation": 0,
      "width": 100,
      "x": 1204,
      "y": 282
    },
    {
      "assetId": "tinyhouse-lamp-lamp-8-upright-tile-4ecf7546",
      "facing": "up-right",
      "flipX": false,
      "height": 92,
      "id": "starter-29-left-desk-lamp",
      "kind": "lamp",
      "label": "left desk lamp",
      "rotation": 0,
      "width": 104,
      "x": 385,
      "y": 299
    },
    {
      "assetId": "tinyhouse-macbook-ani-macbook-1-open-downleft-tile-ccb7166b",
      "facing": "down-left",
      "flipX": false,
      "height": 116,
      "id": "starter-27-left-laptop",
      "kind": "laptop",
      "label": "left laptop",
      "rotation": 0,
      "width": 112,
      "x": 693,
      "y": 464
    },
    {
      "assetId": "tinyhouse-macbook-ani-macbook-1-open-downleft-tile-ccb7166b",
      "facing": "down-left",
      "flipX": false,
      "height": 100,
      "id": "starter-28-right-laptop",
      "kind": "laptop",
      "label": "right laptop",
      "rotation": 0,
      "width": 116,
      "x": 860,
      "y": 549
    },
    {
      "assetId": "tinyhouse-pc-oldimac-downleft-tile-23100ec5",
      "facing": "down-left",
      "flipX": false,
      "height": 132,
      "id": "starter-23-left-monitor",
      "kind": "monitor",
      "label": "left monitor",
      "rotation": 0,
      "width": 184,
      "x": 270,
      "y": 196
    },
    {
      "assetId": "tinyhouse-kitchen-mug-green-ba239aaa",
      "facing": "down-right",
      "flipX": false,
      "height": 32,
      "id": "starter-32-central-green-mug",
      "kind": "mug",
      "label": "central green mug",
      "rotation": 0,
      "width": 36,
      "x": 799,
      "y": 576
    },
    {
      "assetId": "tinyhouse-plants-plant-5-3f1674da",
      "facing": "down-right",
      "flipX": false,
      "height": 75,
      "id": "starter-37-left-desk-plant",
      "kind": "plant",
      "label": "left desk plant",
      "rotation": 0,
      "width": 75,
      "x": 228,
      "y": 224
    },
    {
      "assetId": "tinyhouse-plants-plant-2-ff125b0c",
      "facing": "down-right",
      "flipX": false,
      "height": 144,
      "id": "starter-35-lounge-tall-plant",
      "kind": "plant",
      "label": "lounge tall plant",
      "rotation": 0,
      "width": 144,
      "x": 1051,
      "y": 108
    },
    {
      "assetId": "tinyhouse-carpet-carpet-13-611d921d",
      "facing": "down-right",
      "flipX": false,
      "height": 470,
      "id": "starter-6-center-rug",
      "kind": "rug",
      "label": "center rug",
      "rotation": 0,
      "width": 548,
      "x": 590,
      "y": 430
    },
    {
      "assetId": "tinyhouse-kitchen-kitchen-table-downright-dd1c72f8",
      "facing": "down-right",
      "flipX": false,
      "height": 477,
      "id": "starter-17-central-table",
      "kind": "table",
      "label": "central table",
      "rotation": 0,
      "width": 520,
      "x": 598,
      "y": 387
    }
  ],
  "name": "AI Crew Studio",
  "savedAt": "2026-05-19T11:48:03.972Z",
  "version": 1
}
"##;

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
    seed_workspace_map_config(
        &mut connection,
        workspace_id.as_str(),
        WORKSPACE_MAP_CONFIG_JSON,
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

fn seed_workspace_map_config(
    connection: &mut SqliteConnection,
    workspace_id: &str,
    config_json: &str,
) -> QueryResult<usize> {
    sql_query(
        "
        INSERT INTO workspace_map_configs (workspace_id, config_json)
        VALUES (?1, ?2)
        ON CONFLICT(workspace_id) DO UPDATE SET
            config_json = excluded.config_json,
            updated_at = CURRENT_TIMESTAMP
        ",
    )
    .bind::<Text, _>(workspace_id)
    .bind::<Text, _>(config_json)
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
            current_x = excluded.current_x,
            current_y = excluded.current_y,
            current_facing = excluded.current_facing,
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
