

table! {
    authentication (id) {
        id -> Int4,
        uid -> Int4,
        login_type -> Int4,
        account -> Varchar,
        credential -> Nullable<Varchar>,
    }
}

table! {
    persons (id) {
        id -> Int4,
        uid -> Int4,
        nick_name -> Varchar,
        is_disabled -> Bool,
        is_admin -> Bool,
        extra -> Nullable<Jsonb>,
    }
}

allow_tables_to_appear_in_same_query!(
    authentication,
    persons,
);




