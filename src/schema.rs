diesel::table! {
    lemmy.credentials (domain, username) {
        domain -> Text,
        username -> Text,
        password -> Text,
    }
}

diesel::table! {
    lemmy.post_reports (domain, id) {
        domain -> Text,
        id -> Int4,
        data -> Jsonb,
    }
}

diesel::table! {
    lemmy.comment_reports (domain, id) {
        domain -> Text,
        id -> Int4,
        data -> Jsonb,
    }
}