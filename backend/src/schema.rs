// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        email -> Text,
        display_name -> Nullable<Text>,
        created_at -> Timestamp,
    }
}
