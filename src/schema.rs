table! {
    records (id) {
        id -> Nullable<Integer>,
        chat_type -> Text,
        owner_id -> Text,
        group_id -> Text,
        sender -> Text,
        content -> Text,
        timestamp -> BigInt,
        metadata -> Nullable<Binary>,
    }
}
