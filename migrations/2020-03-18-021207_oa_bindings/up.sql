create table oa_bindings
(
    id           serial  not null
        constraint oa_bindings_pkey
            primary key,
    uid          integer not null
        constraint oa_bindings_persons_uid_fk
            references persons (uid)
            on update restrict,
    student_id   char(10),
    oa_password  char(10),
    oa_certified boolean not null default false,
    class        char(8)
);