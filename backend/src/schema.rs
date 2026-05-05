// @generated automatically by Diesel CLI.

diesel::table! {
    conversations (id) {
        id -> Text,
        user_id -> Integer,
        workspace_id -> Text,
        minion_id -> Nullable<Text>,
        title -> Text,
        codex_thread_id -> Nullable<Text>,
        current_session_id -> Nullable<Text>,
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
        conversation_id -> Text,
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
    minion_elements (id) {
        id -> Text,
        minion_id -> Text,
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
    minions (id) {
        id -> Text,
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

diesel::joinable!(conversations -> minions (minion_id));
diesel::joinable!(conversations -> users (user_id));
diesel::joinable!(conversations -> workspaces (workspace_id));
diesel::joinable!(messages -> conversations (conversation_id));
diesel::joinable!(minion_elements -> minions (minion_id));
diesel::joinable!(minions -> workspaces (workspace_id));
diesel::joinable!(workspaces -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    conversations,
    messages,
    minion_elements,
    minions,
    users,
    workspaces,
);
