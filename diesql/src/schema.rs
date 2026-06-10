// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}

diesel::table! {
    users (uid) {
        uid -> Int4,
        username -> Varchar,
        password -> Text,
        manager -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(posts, users,);
