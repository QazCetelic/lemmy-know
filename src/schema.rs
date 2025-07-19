diesel::table! {
    credentials (domain, username) {
        domain -> Text,
        username -> Text,
        password -> Text,
    }
}

diesel::table! {
    post_reports (domain, id) {
        domain -> Text,
        id -> Int4,
        data -> Jsonb,
    }
}

diesel::table! {
    comment_reports (domain, id) {
        domain -> Text,
        id -> Int4,
        data -> Jsonb,
    }
}