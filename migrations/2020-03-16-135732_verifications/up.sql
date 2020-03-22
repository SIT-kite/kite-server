create table verifications
(
    id         serial      not null
        constraint verifications_pkey
            primary key,
    uid        serial      not null
        constraint verifications_persons_uid_fk
            references persons (uid)
            on update restrict,
    login_type smallint    not null,
    account    varchar(40) not null,
    credential varchar(40),
);