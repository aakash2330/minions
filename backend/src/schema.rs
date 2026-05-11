// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Text,
        session_id -> Text,
        role -> Text,
        text -> Text,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    sessions (session_id) {
        session_id -> Text,
        workspace_id -> Text,
        name -> Text,
        kind -> Text,
        status -> Text,
        spawn_x -> Integer,
        spawn_y -> Integer,
        spawn_facing -> Text,
        current_x -> Integer,
        current_y -> Integer,
        current_facing -> Text,
        codex_thread_id -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        archived_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    workspace_elements (id) {
        id -> Text,
        workspace_id -> Text,
        assigned_session_id -> Nullable<Text>,
        kind -> Text,
        label -> Text,
        position_x -> Integer,
        position_y -> Integer,
        facing -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Text,
        name -> Text,
        root_path -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(messages -> sessions (session_id));
diesel::joinable!(sessions -> workspaces (workspace_id));
diesel::joinable!(workspace_elements -> sessions (assigned_session_id));
diesel::joinable!(workspace_elements -> workspaces (workspace_id));

diesel::allow_tables_to_appear_in_same_query!(messages, sessions, workspace_elements, workspaces,);
