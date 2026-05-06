// @generated automatically by Diesel CLI.

diesel::table! {
    threads (id) {
        id -> Text,
        user_id -> Integer,
        workspace_id -> Text,
        session_id -> Nullable<Text>,
        title -> Text,
        codex_thread_id -> Nullable<Text>,
        cwd -> Nullable<Text>,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        archived_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    messages (id) {
        id -> Text,
        thread_id -> Text,
        role -> Text,
        text -> Text,
        status -> Text,
        codex_turn_id -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    session_elements (id) {
        id -> Text,
        session_id -> Text,
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
        created_at -> Timestamp,
        updated_at -> Timestamp,
        archived_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        email -> Text,
        display_name -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    workspaces (id) {
        id -> Text,
        user_id -> Integer,
        name -> Text,
        root_path -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(messages -> threads (thread_id));
diesel::joinable!(session_elements -> sessions (session_id));
diesel::joinable!(sessions -> workspaces (workspace_id));
diesel::joinable!(threads -> sessions (session_id));
diesel::joinable!(threads -> users (user_id));
diesel::joinable!(threads -> workspaces (workspace_id));
diesel::joinable!(workspaces -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    session_elements,
    sessions,
    threads,
    users,
    workspaces,
);
