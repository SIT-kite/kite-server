table! {
    oa_bindings (id) {
        id -> Int4,
        uid -> Int4,
        student_id -> Nullable<Bpchar>,
        oa_password -> Nullable<Bpchar>,
        oa_certified -> Bool,
        class -> Nullable<Bpchar>,
    }
}

table! {
    persons (id) {
        id -> Int4,
        uid -> Int4,
        sex -> Int4,
        real_name -> Nullable<Varchar>,
        nick_name -> Nullable<Varchar>,
        avatar_url -> Nullable<Varchar>,
        avatar -> Nullable<Varchar>,
        profile -> Nullable<Varchar>,
        status -> Int4,
        country -> Nullable<Varchar>,
        province -> Nullable<Varchar>,
        city -> Nullable<Varchar>,
        role -> Int2,
    }
}

table! {
    verifications (id) {
        id -> Int4,
        uid -> Int4,
        login_type -> Int4,
        account -> Varchar,
        credential -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    oa_bindings,
    persons,
    verifications,
);
