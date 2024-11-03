table! {
    attachments (id) {
        id -> Nullable<Integer>,
        record_id -> Integer,
        name -> Text,
        hash -> BigInt,
    }
}

table! {
    blobs (hash) {
        hash -> BigInt,
        blob -> Binary,
    }
}

table! {
    records (id) {
        id -> Nullable<Integer>,
        chat_type -> Text,
        owner_id -> Text,
        group_id -> Text,
        sender_id -> Text,
        sender_name -> Text,
        content -> Text,
        timestamp -> BigInt,
        metadata -> Nullable<Binary>,
    }
}

allow_tables_to_appear_in_same_query!(
    attachments,
    blobs,
    records,
);
