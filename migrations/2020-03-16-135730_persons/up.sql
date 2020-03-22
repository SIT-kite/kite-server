create table persons
(
    id           serial not null
        constraint person_pkey
            primary key,
    uid          serial not null
        constraint persons_uid_key
            unique,
    sex          int not null default 0,
    real_name    varchar(20),
    nick_name    varchar(20),
    avatar_url   varchar(100),
    avatar       varchar(64),
    profile      varchar(50),
    status       int not null,
    country      varchar(30),
    province     varchar(30),
    city         varchar(30),
    role       int    not null
);