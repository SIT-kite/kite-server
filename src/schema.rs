use crate::sign::schema::*;
use crate::user::schema::*;

table! {
    attachments (id) {
        id -> Int4,
        filename -> Varchar,
        uploader -> Int4,
        upload_time -> Timestamp,
        is_deleted -> Nullable<Bool>,
    }
}

joinable!(event_applicants -> persons (uid));
joinable!(event_applicants -> events (event_id));
joinable!(attachments -> persons (uploader));

allow_tables_to_appear_in_same_query!(event_applicants, persons, events);
allow_tables_to_appear_in_same_query!(attachments, persons);
