//! The content is generated automatically by diesel-cli
//!
//! Command:
//!  diesel print-schema --schema sign

table! {
    sign.event_applicants (id) {
        id -> Int4,
        uid -> Int4,
        event_id -> Int4,
        apply_time -> Timestamp,
        sign_time -> Nullable<Timestamp>,
        sign_type -> Nullable<Int4>,
    }
}

table! {
    sign.events (id) {
        id -> Int4,
        event_id -> Int4,
        publisher_uid -> Int4,
        title -> Varchar,
        description -> Text,
        start_time -> Timestamp,
        end_time -> Nullable<Timestamp>,
        create_time -> Timestamp,
        tags -> Nullable<Array<Varchar>>,
        deleted -> Bool,
        max_people -> Nullable<Int2>,
        place -> Varchar,
        image -> Varchar,
        attachments -> Nullable<Array<Int4>>,
    }
}


allow_tables_to_appear_in_same_query!(
    event_applicants,
    events,
);

