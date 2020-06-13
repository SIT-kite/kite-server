use crate::sign::schema::*;
use crate::user::schema::*;

joinable!(event_applicants -> persons (uid));
joinable!(event_applicants -> events (event_id));

allow_tables_to_appear_in_same_query!(event_applicants, persons, events);
