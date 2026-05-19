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
    asset_id: Option<&'static str>,
    width: Option<i32>,
    height: Option<i32>,
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
        asset_id: None,
        width: None,
        height: None,
    },
    WorkspaceElementSeed {
        id: "bob-workdesk",
        assigned_session_id: Some("bob"),
        kind: WorkspaceElementKind::PersonalTable,
        label: "personal table",
        position_x: 1155,
        position_y: 420,
        facing: Direction::UpRight,
        asset_id: None,
        width: None,
        height: None,
    },
    WorkspaceElementSeed {
        id: "kevin-meeting-table",
        assigned_session_id: Some("kevin"),
        kind: WorkspaceElementKind::MeetingTable,
        label: "meeting table",
        position_x: 720,
        position_y: 645,
        facing: Direction::UpRight,
        asset_id: None,
        width: None,
        height: None,
    },
    WorkspaceElementSeed {
        id: "bob-meeting-table",
        assigned_session_id: Some("bob"),
        kind: WorkspaceElementKind::MeetingTable,
        label: "meeting table",
        position_x: 900,
        position_y: 705,
        facing: Direction::UpRight,
        asset_id: None,
        width: None,
        height: None,
    },
    WorkspaceElementSeed {
        id: "starter-6-center-rug",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Rug,
        label: "center rug",
        position_x: 590,
        position_y: 430,
        facing: Direction::DownRight,
        asset_id: Some("Carpet_13.png"),
        width: Some(548),
        height: Some(470),
    },
    WorkspaceElementSeed {
        id: "starter-10-left-stool",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Stool,
        label: "left stool",
        position_x: 540,
        position_y: 520,
        facing: Direction::DownRight,
        asset_id: Some("Kitchen_Stool.png"),
        width: Some(86),
        height: Some(86),
    },
    WorkspaceElementSeed {
        id: "starter-12-front-left-stool",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Stool,
        label: "front left stool",
        position_x: 650,
        position_y: 668,
        facing: Direction::DownRight,
        asset_id: Some("Kitchen_Stool.png"),
        width: Some(128),
        height: Some(116),
    },
    WorkspaceElementSeed {
        id: "starter-13-front-right-stool",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Stool,
        label: "front right stool",
        position_x: 716,
        position_y: 660,
        facing: Direction::DownRight,
        asset_id: Some("Kitchen_Stool.png"),
        width: Some(188),
        height: Some(156),
    },
    WorkspaceElementSeed {
        id: "starter-14-left-desk",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Desk,
        label: "left desk",
        position_x: 72,
        position_y: 84,
        facing: Direction::DownRight,
        asset_id: Some("Desk_1_DownRight_Tile.png"),
        width: Some(624),
        height: Some(536),
    },
    WorkspaceElementSeed {
        id: "starter-15-right-desk",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Desk,
        label: "right desk",
        position_x: 1032,
        position_y: 100,
        facing: Direction::DownRight,
        asset_id: Some("Desk_1_DownRight_Tile.png"),
        width: Some(568),
        height: Some(564),
    },
    WorkspaceElementSeed {
        id: "starter-17-central-table",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Table,
        label: "central table",
        position_x: 602,
        position_y: 387,
        facing: Direction::DownRight,
        asset_id: Some("Kitchen_Table_DownRight.png"),
        width: Some(520),
        height: Some(477),
    },
    WorkspaceElementSeed {
        id: "starter-18-lounge-sofa",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Sofa,
        label: "lounge sofa",
        position_x: 577,
        position_y: 0,
        facing: Direction::DownRight,
        asset_id: Some("Sofa_3_DownRight_Tile.png"),
        width: Some(532),
        height: Some(432),
    },
    WorkspaceElementSeed {
        id: "starter-23-left-monitor",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Monitor,
        label: "left monitor",
        position_x: 270,
        position_y: 196,
        facing: Direction::DownLeft,
        asset_id: Some("OldImac_DownLeft_Tile.png"),
        width: Some(184),
        height: Some(132),
    },
    WorkspaceElementSeed {
        id: "starter-25-left-keyboard",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Keyboard,
        label: "left keyboard",
        position_x: 256,
        position_y: 286,
        facing: Direction::DownRight,
        asset_id: Some("OldKeyboard_DownRight_Tile.png"),
        width: Some(104),
        height: Some(84),
    },
    WorkspaceElementSeed {
        id: "starter-26-right-keyboard",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Keyboard,
        label: "right keyboard",
        position_x: 1204,
        position_y: 282,
        facing: Direction::DownRight,
        asset_id: Some("OldKeyboard_DownRight_Tile.png"),
        width: Some(100),
        height: Some(56),
    },
    WorkspaceElementSeed {
        id: "starter-27-left-laptop",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Laptop,
        label: "left laptop",
        position_x: 693,
        position_y: 464,
        facing: Direction::DownLeft,
        asset_id: Some("Macbook_1_Open_DownLeft_Tile.png"),
        width: Some(112),
        height: Some(116),
    },
    WorkspaceElementSeed {
        id: "starter-28-right-laptop",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Laptop,
        label: "right laptop",
        position_x: 860,
        position_y: 549,
        facing: Direction::DownLeft,
        asset_id: Some("Macbook_1_Open_DownLeft_Tile.png"),
        width: Some(116),
        height: Some(100),
    },
    WorkspaceElementSeed {
        id: "starter-29-left-desk-lamp",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Lamp,
        label: "left desk lamp",
        position_x: 385,
        position_y: 299,
        facing: Direction::UpRight,
        asset_id: Some("Lamp_8_UpRight_Tile.png"),
        width: Some(104),
        height: Some(92),
    },
    WorkspaceElementSeed {
        id: "starter-31-central-book-stack",
        assigned_session_id: None,
        kind: WorkspaceElementKind::BookStack,
        label: "central book stack",
        position_x: 832,
        position_y: 521,
        facing: Direction::DownRight,
        asset_id: Some("Book_8.png"),
        width: Some(66),
        height: Some(66),
    },
    WorkspaceElementSeed {
        id: "starter-32-central-green-mug",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Mug,
        label: "central green mug",
        position_x: 799,
        position_y: 576,
        facing: Direction::DownRight,
        asset_id: Some("Mug_Green.png"),
        width: Some(36),
        height: Some(32),
    },
    WorkspaceElementSeed {
        id: "starter-35-lounge-tall-plant",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Plant,
        label: "lounge tall plant",
        position_x: 1051,
        position_y: 108,
        facing: Direction::DownRight,
        asset_id: Some("Plant_2.png"),
        width: Some(144),
        height: Some(144),
    },
    WorkspaceElementSeed {
        id: "starter-37-left-desk-plant",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Plant,
        label: "left desk plant",
        position_x: 228,
        position_y: 224,
        facing: Direction::DownRight,
        asset_id: Some("Plant_5.png"),
        width: Some(75),
        height: Some(75),
    },
    WorkspaceElementSeed {
        id: "starter-38-right-desk-cactus",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Cactus,
        label: "right desk cactus",
        position_x: 1198,
        position_y: 209,
        facing: Direction::DownRight,
        asset_id: Some("Cactus_1.png"),
        width: Some(75),
        height: Some(75),
    },
    WorkspaceElementSeed {
        id: "c3acc4ad-0e36-4ce0-89ee-aaf90a186661",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Chair,
        label: "left gaming chair",
        position_x: 160,
        position_y: 344,
        facing: Direction::UpRight,
        asset_id: Some("GChair_9_UpRight.png"),
        width: Some(200),
        height: Some(88),
    },
    WorkspaceElementSeed {
        id: "47a40804-fd9b-4cc8-a044-1ecd5920b9be",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Chair,
        label: "right chair",
        position_x: 1080,
        position_y: 336,
        facing: Direction::UpRight,
        asset_id: Some("Chair_2_UpRight_Tile.png"),
        width: Some(224),
        height: Some(168),
    },
    WorkspaceElementSeed {
        id: "49c929d1-cb2e-4a83-9cc0-f035ba6d4751",
        assigned_session_id: None,
        kind: WorkspaceElementKind::Computer,
        label: "right computer",
        position_x: 1232,
        position_y: 192,
        facing: Direction::DownLeft,
        asset_id: Some("OldPC_DownLeft_Tile.png"),
        width: Some(164),
        height: Some(144),
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
            facing,
            asset_id,
            width,
            height
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(id) DO UPDATE SET
            workspace_id = excluded.workspace_id,
            assigned_session_id = excluded.assigned_session_id,
            kind = excluded.kind,
            label = excluded.label,
            position_x = excluded.position_x,
            position_y = excluded.position_y,
            facing = excluded.facing,
            asset_id = excluded.asset_id,
            width = excluded.width,
            height = excluded.height,
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
    .bind::<diesel::sql_types::Nullable<Text>, _>(element.asset_id)
    .bind::<diesel::sql_types::Nullable<Integer>, _>(element.width)
    .bind::<diesel::sql_types::Nullable<Integer>, _>(element.height)
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
