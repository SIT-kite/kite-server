--
-- PostgreSQL database dump
--

-- Dumped from database version 13.2 (Ubuntu 13.2-1.pgdg20.04+1)
-- Dumped by pg_dump version 13.2 (Ubuntu 13.2-1.pgdg20.04+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: kite; Type: DATABASE; Schema: -; Owner: postgres
--

CREATE DATABASE kite WITH TEMPLATE = template0 ENCODING = 'UTF8' LOCALE = 'en_US.UTF-8';


ALTER DATABASE kite OWNER TO postgres;

\connect kite

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: kite; Type: DATABASE PROPERTIES; Schema: -; Owner: postgres
--

ALTER DATABASE kite SET "TimeZone" TO 'Asia/Shanghai';


\connect kite

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: base; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA base;


ALTER SCHEMA base OWNER TO postgres;

--
-- Name: SCHEMA base; Type: COMMENT; Schema: -; Owner: postgres
--

COMMENT ON SCHEMA base IS '基础数据';


--
-- Name: checking; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA checking;


ALTER SCHEMA checking OWNER TO postgres;

--
-- Name: SCHEMA checking; Type: COMMENT; Schema: -; Owner: postgres
--

COMMENT ON SCHEMA checking IS '返校码';


--
-- Name: dormitory; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA dormitory;


ALTER SCHEMA dormitory OWNER TO postgres;

--
-- Name: edu; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA edu;


ALTER SCHEMA edu OWNER TO postgres;

--
-- Name: SCHEMA edu; Type: COMMENT; Schema: -; Owner: postgres
--

COMMENT ON SCHEMA edu IS '课程列表和选课相关';


--
-- Name: events; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA events;


ALTER SCHEMA events OWNER TO postgres;

--
-- Name: SCHEMA events; Type: COMMENT; Schema: -; Owner: postgres
--

COMMENT ON SCHEMA events IS '签到和活动相关';


--
-- Name: freshman; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA freshman;


ALTER SCHEMA freshman OWNER TO postgres;

--
-- Name: SCHEMA freshman; Type: COMMENT; Schema: -; Owner: postgres
--

COMMENT ON SCHEMA freshman IS '迎新相关表项';


--
-- Name: mall; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA mall;


ALTER SCHEMA mall OWNER TO postgres;

--
-- Name: search; Type: SCHEMA; Schema: -; Owner: postgres
--

CREATE SCHEMA search;


ALTER SCHEMA search OWNER TO postgres;

--
-- Name: SCHEMA search; Type: COMMENT; Schema: -; Owner: postgres
--

COMMENT ON SCHEMA search IS '小风筝搜索模块数据';


--
-- Name: zhparser; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS zhparser WITH SCHEMA public;


--
-- Name: EXTENSION zhparser; Type: COMMENT; Schema: -; Owner:
--

COMMENT ON EXTENSION zhparser IS 'a parser for full-text search of Chinese';


--
-- Name: roomrank; Type: TYPE; Schema: dormitory; Owner: postgres
--

CREATE TYPE dormitory.roomrank AS
(
    room        integer,
    consumption real,
    rank        integer,
    room_count  integer
);


ALTER TYPE dormitory.roomrank OWNER TO postgres;

--
-- Name: freshmanstudent; Type: TYPE; Schema: freshman; Owner: postgres
--

CREATE TYPE freshman.freshmanstudent AS
(
    name           text,
    uid            integer,
    student_id     text,
    college        text,
    major          text,
    campus         text,
    building       text,
    room           integer,
    bed            text,
    counselor_name text,
    counselor_tel  text,
    visible        boolean,
    secret         text
);


ALTER TYPE freshman.freshmanstudent OWNER TO postgres;

--
-- Name: get_consumption_report_by_day(date, date, integer); Type: FUNCTION; Schema: dormitory; Owner: postgres
--

CREATE FUNCTION dormitory.get_consumption_report_by_day(start_date date, end_date date, room_to_query integer)
    RETURNS TABLE
            (
                day            text,
                charged_amount real,
                used_amount    real
            )
    LANGUAGE plpgsql
    STABLE STRICT
AS
$$
BEGIN
    RETURN QUERY WITH statistic_series AS (
        SELECT to_char(ts, 'YYYY-MM-DD') AS _day, amount
        FROM dormitory.consumption
        WHERE ts BETWEEN start_date AND end_date
          AND room = room_to_query)
                 SELECT DISTINCT ss._day                                  AS day,
                                 ABS(positive_consumption.charged_amount) AS charged_amount,
                                 ABS(negative_consumption.used_amount)    AS used_amount
                 FROM statistic_series ss
                          LEFT JOIN (
                     SELECT _day, SUM(amount) AS charged_amount
                     FROM statistic_series
                     WHERE amount > 0
                     GROUP BY _day
                 ) AS positive_consumption
                                    ON ss._day = positive_consumption._day
                          LEFT JOIN (
                     SELECT _day, SUM(amount) AS used_amount
                     FROM statistic_series
                     WHERE amount < 0
                     GROUP BY _day
                 ) AS negative_consumption
                                    ON ss._day = negative_consumption._day
                 ORDER BY DAY;
END;
$$;


ALTER FUNCTION dormitory.get_consumption_report_by_day(start_date date, end_date date, room_to_query integer) OWNER TO postgres;

--
-- Name: get_consumption_report_by_hour(timestamp with time zone, timestamp with time zone, integer); Type: FUNCTION; Schema: dormitory; Owner: postgres
--

CREATE FUNCTION dormitory.get_consumption_report_by_hour(start_date timestamp with time zone,
                                                         end_date timestamp with time zone, room_to_query integer)
    RETURNS TABLE
            (
                hour           text,
                charged_amount real,
                used_amount    real
            )
    LANGUAGE plpgsql
    STABLE STRICT
AS
$$
BEGIN
    RETURN QUERY WITH statistic_series AS (
        SELECT to_char(ts AT TIME ZONE 'Asia/Shanghai', 'YYYY-MM-DD HH24:00') AS _hour, amount
        FROM dormitory.consumption
        WHERE ts BETWEEN start_date AND end_date
          AND room = room_to_query)
                 SELECT DISTINCT ss._hour                                 AS day,
                                 ABS(positive_consumption.charged_amount) AS charged_amount,
                                 ABS(negative_consumption.used_amount)    AS used_amount
                 FROM statistic_series ss
                          LEFT JOIN (
                     SELECT _hour, SUM(amount) AS charged_amount
                     FROM statistic_series
                     WHERE amount > 0
                     GROUP BY _hour
                 ) AS positive_consumption
                                    ON ss._hour = positive_consumption._hour
                          LEFT JOIN (
                     SELECT _hour, SUM(amount) AS used_amount
                     FROM statistic_series
                     WHERE amount < 0
                     GROUP BY _hour
                 ) AS negative_consumption
                                    ON ss._hour = negative_consumption._hour
                 ORDER BY DAY;
END;
$$;


ALTER FUNCTION dormitory.get_consumption_report_by_hour(start_date timestamp with time zone, end_date timestamp with time zone, room_to_query integer) OWNER TO postgres;

--
-- Name: get_consumption_report_by_hour2(timestamp with time zone, timestamp with time zone, integer); Type: FUNCTION; Schema: dormitory; Owner: postgres
--

CREATE FUNCTION dormitory.get_consumption_report_by_hour2(start_date timestamp with time zone,
                                                          end_date timestamp with time zone, room_to_query integer)
    RETURNS TABLE
            (
                hour           text,
                charged_amount real,
                used_amount    real
            )
    LANGUAGE plpgsql
    STABLE STRICT
AS
$$
BEGIN
    RETURN QUERY WITH statistic_series AS (
        SELECT to_char(ts, 'YYYY-MM-DD HH24:00 UTC') AS _hour, amount
        FROM dormitory.consumption
        WHERE ts BETWEEN start_date AND end_date
          AND room = room_to_query)
                 SELECT DISTINCT ss._hour                                 AS day,
                                 ABS(positive_consumption.charged_amount) AS charged_amount,
                                 ABS(negative_consumption.used_amount)    AS used_amount
                 FROM statistic_series ss
                          LEFT JOIN (
                     SELECT _hour, SUM(amount) AS charged_amount
                     FROM statistic_series
                     WHERE amount > 0
                     GROUP BY _hour
                 ) AS positive_consumption
                                    ON ss._hour = positive_consumption._hour
                          LEFT JOIN (
                     SELECT _hour, SUM(amount) AS used_amount
                     FROM statistic_series
                     WHERE amount < 0
                     GROUP BY _hour
                 ) AS negative_consumption
                                    ON ss._hour = negative_consumption._hour
                 ORDER BY DAY;
END;
$$;


ALTER FUNCTION dormitory.get_consumption_report_by_hour2(start_date timestamp with time zone, end_date timestamp with time zone, room_to_query integer) OWNER TO postgres;

--
-- Name: get_room_24hour_rank(integer); Type: FUNCTION; Schema: dormitory; Owner: postgres
--

CREATE FUNCTION dormitory.get_room_24hour_rank(room_id integer) RETURNS dormitory.roomrank
    LANGUAGE plpgsql
    STABLE
AS
$$
DECLARE
    result     RECORD;
    room_count integer;
BEGIN
    SELECT COUNT(*) INTO room_count FROM dormitory.rooms;

    SELECT room, consumption, rank, room_count
    INTO result
    FROM dormitory.rank_last_24hour_consumption()
    WHERE room = room_id;

    IF NOT FOUND THEN
        SELECT room_id           AS room,
               CAST(0.0 AS real) AS consumption,
               room_count        AS rank,
               room_count
        INTO result;
    END IF;

    RETURN result;
END;
$$;


ALTER FUNCTION dormitory.get_room_24hour_rank(room_id integer) OWNER TO postgres;

--
-- Name: rank_last_24hour_consumption(); Type: FUNCTION; Schema: dormitory; Owner: postgres
--

CREATE FUNCTION dormitory.rank_last_24hour_consumption()
    RETURNS TABLE
            (
                room        integer,
                consumption real,
                rank        integer
            )
    LANGUAGE plpgsql
    STABLE STRICT
AS
$$
DECLARE
    cur_ts timestamp;
BEGIN
    SELECT current_timestamp AT TIME ZONE 'Asia/ShangHai' INTO cur_ts;

    RETURN QUERY
        SELECT rank_table.room,
               rank_table.consumption,
               CAST(rank() OVER (ORDER BY rank_table.consumption DESC) AS integer) AS rank
        FROM (
                 SELECT c.room, ABS(SUM(amount)) AS consumption
                 FROM dormitory.consumption c
                 WHERE ts BETWEEN cur_ts - '1 day'::interval AND cur_ts
                   AND amount < 0
                 GROUP BY c.room
             ) rank_table;
END
$$;


ALTER FUNCTION dormitory.rank_last_24hour_consumption() OWNER TO postgres;

--
-- Name: query_student(text, text); Type: FUNCTION; Schema: freshman; Owner: postgres
--

CREATE FUNCTION freshman.query_student(query_str text, secret_str text) RETURNS freshman.freshmanstudent
    LANGUAGE plpgsql
    STRICT
AS
$$
DECLARE
    result freshman.FreshmanStudent;

BEGIN
    SELECT name,
           uid,
           student_id,
           college,
           major,
           campus,
           building,
           room,
           bed,
           counselor_name,
           counselor_tel,
           visible,
           secret
    INTO result
    FROM freshman.students
    WHERE (name = query_str OR student_id = query_str OR ticket = query_str)
      AND secret = secret_str;

    RETURN result;
END;
$$;


ALTER FUNCTION freshman.query_student(query_str text, secret_str text) OWNER TO postgres;

--
-- Name: record_contact_change(); Type: FUNCTION; Schema: freshman; Owner: postgres
--

CREATE FUNCTION freshman.record_contact_change() RETURNS trigger
    LANGUAGE plpgsql
AS
$$
BEGIN
    IF old.contact <> new.contact THEN
        INSERT INTO freshman.change_log(student_id, contact) VALUES (old.student_id, old.contact);
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION freshman.record_contact_change() OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: goods; Type: TABLE; Schema: mall; Owner: postgres
--

CREATE TABLE mall.goods
(
    id           integer                                       NOT NULL,
    title        text                                          NOT NULL,
    description  text,
    status       smallint                                      NOT NULL,
    cover_image  text                                          NOT NULL,
    campus       text                                          NOT NULL,
    images       text[]                   DEFAULT '{}'::text[] NOT NULL,
    tags         text[]                   DEFAULT '{}'::text[] NOT NULL,
    price        real                                          NOT NULL,
    publisher    integer                                       NOT NULL,
    publish_time timestamp with time zone DEFAULT now()        NOT NULL,
    wish         smallint                 DEFAULT 0            NOT NULL,
    views        integer                  DEFAULT 0            NOT NULL,
    sort         integer                                       NOT NULL,
    features     jsonb                    DEFAULT '{}'::jsonb  NOT NULL
);


ALTER TABLE mall.goods
    OWNER TO postgres;

--
-- Name: TABLE goods; Type: COMMENT; Schema: mall; Owner: postgres
--

COMMENT ON TABLE mall.goods IS '商品列表';


--
-- Name: query_goods(text, integer); Type: FUNCTION; Schema: mall; Owner: postgres
--

CREATE FUNCTION mall.query_goods(query_string text, sort_id integer) RETURNS SETOF mall.goods
    LANGUAGE plpgsql
AS
$$
BEGIN
    -- If query string is not null, search title for it.

    IF query_string IS NULL THEN
        CREATE TEMPORARY TABLE result AS
        SELECT * FROM mall.goods;
    ELSE
        CREATE TEMPORARY TABLE result AS
        SELECT * FROM mall.goods WHERE title LIKE '%' || query_string || '%';
    END IF;

    -- If sort is null, get all of the last result.
    IF sort_id IS NULL THEN
        RETURN QUERY SELECT * FROM result;
    ELSE
        RETURN QUERY SELECT * FROM result WHERE sort = sort_id;
    END IF;

END;
$$;


ALTER FUNCTION mall.query_goods(query_string text, sort_id integer) OWNER TO postgres;

--
-- Name: calc_consumption(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.calc_consumption() RETURNS trigger
    LANGUAGE plpgsql
AS
$$
BEGIN
    -- If the room has no record before, do insert.
    IF old IS NULL THEN
        INSERT INTO dormitory.consumption (room, amount) VALUES (new.room, new.total_balance);
    ELSE
        -- Check whether it's changed or not.
        IF ABS(new.total_balance - old.total_balance) > 0.01 THEN
            INSERT INTO dormitory.consumption (room, amount) VALUES (new.room, new.total_balance - old.total_balance);
        ELSE
            old.ts = current_timestamp; -- Set update time
            RETURN old;
        END IF;
    END IF;

    RETURN NEW;
END;
$$;


ALTER FUNCTION public.calc_consumption() OWNER TO postgres;

--
-- Name: record_authentication_change(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.record_authentication_change() RETURNS trigger
    LANGUAGE plpgsql
AS
$$
BEGIN
    IF OLD.uid IS NOT NULL THEN
        INSERT INTO public.authentication_log(uid, account, credential) VALUES (OLD.uid, OLD.account, OLD.credential);
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.record_authentication_change() OWNER TO postgres;

--
-- Name: search_notice(text); Type: FUNCTION; Schema: search; Owner: postgres
--

CREATE FUNCTION search.search_notice(query_string text)
    RETURNS TABLE
            (
                rank         double precision,
                url          text,
                title        text,
                publish_time timestamp with time zone,
                department   text,
                author       text,
                sort         text,
                content      text
            )
    LANGUAGE plpgsql
    STRICT
AS
$$
DECLARE
    query_v tsquery;
BEGIN
    query_v := to_tsquery('zh_cfg', query_string);

    RETURN QUERY SELECT ts_rank_cd(to_tsvector(n.title), query_v, 32)
                            + 3 * ts_rank(to_tsvector(n.content), query_v) AS rank,
                        n.url,
                        ts_headline('zh_cfg', n.title, query_v),
                        n.publish_time,
                        n.department,
                        n.author,
                        n.sort,
                        ts_headline('zh_cfg', left(n.content, 200), query_v)
                 FROM search.notices n
                 WHERE query_v @@ to_tsvector('zh_cfg', n.title || n.content)
                 ORDER BY rank DESC;
END
$$;


ALTER FUNCTION search.search_notice(query_string text) OWNER TO postgres;

--
-- Name: search_page(text); Type: FUNCTION; Schema: search; Owner: postgres
--

CREATE FUNCTION search.search_page(query_string text)
    RETURNS TABLE
            (
                rank         double precision,
                title        text,
                host         text,
                path         text,
                publish_date date,
                update_date  date,
                link_count   smallint,
                content      text,
                disable      boolean
            )
    LANGUAGE plpgsql
    STRICT
AS
$$
DECLARE
    query_v tsquery;
BEGIN
    query_v := to_tsquery('kite_web', query_string);

    RETURN QUERY SELECT ts_rank_cd(to_tsvector(n.title), query_v, 32)
                            + 3 * ts_rank(to_tsvector(n.content), query_v) AS rank,
                        ts_headline('kite_web', n.title, query_v),
                        n.host,
                        n.path,
                        n.publish_date,
                        n.update_date,
                        n.link_count,
                        ts_headline('kite_web', left(n.content, 200), query_v)
                 FROM search.pages n
                 WHERE query_v @@ to_tsvector('kite_web', n.title || n.content)
                 ORDER BY rank DESC;
END
$$;


ALTER FUNCTION search.search_page(query_string text) OWNER TO postgres;

--
-- Name: submit_attachment(text, text, text, text, integer, text, text, text); Type: PROCEDURE; Schema: search; Owner: postgres
--

CREATE PROCEDURE search.submit_attachment(_title text, _host text, _path text, _ext text, _size integer,
                                          _local_name text, _checksum text, _referer text)
    LANGUAGE plpgsql
AS
$$
BEGIN
    INSERT INTO search.attachments (title, host, path, ext, size, local_name, checksum, referer)
    VALUES (_title, _host, _path, _ext, _size, _local_name, _checksum, _referer)
    ON CONFLICT (HASHTEXT(host || path))
        DO UPDATE
        SET title      = _title,
            ext        = _ext,
            size       = _size,
            local_name = _local_name,
            checksum   = _checksum,
            referer    = _referer;
END;
$$;


ALTER PROCEDURE search.submit_attachment(_title text, _host text, _path text, _ext text, _size integer, _local_name text, _checksum text, _referer text) OWNER TO postgres;

--
-- Name: submit_page(text, text, text, date, integer, text); Type: PROCEDURE; Schema: search; Owner: postgres
--

CREATE PROCEDURE search.submit_page(_title text, _host text, _path text, _publish_date date, _link_count integer,
                                    _content text)
    LANGUAGE plpgsql
AS
$$
BEGIN
    INSERT INTO search.pages (title, host, path, publish_date, link_count, content)
    VALUES (_title, _host, _path, _publish_date, _link_count, _content)
    ON CONFLICT (HASHTEXT(host || path))
        DO UPDATE
        SET title        = _title,
            publish_date = _publish_date,
            link_count   = _link_count,
            content      = _content,
            update_date  = current_date;
END;
$$;


ALTER PROCEDURE search.submit_page(_title text, _host text, _path text, _publish_date date, _link_count integer, _content text) OWNER TO postgres;

--
-- Name: update_page(text, text, text, text); Type: PROCEDURE; Schema: search; Owner: postgres
--

CREATE PROCEDURE search.update_page(_host text, _path text, _title text, _content text)
    LANGUAGE plpgsql
AS
$$
BEGIN
    UPDATE search.pages
    SET title       = _title,
        content     = _content,
        update_date = current_date
    WHERE (hashtext(host || path)) = hashtext((_host || _path));
END;
$$;


ALTER PROCEDURE search.update_page(_host text, _path text, _title text, _content text) OWNER TO postgres;

--
-- Name: zh_cfg; Type: TEXT SEARCH CONFIGURATION; Schema: public; Owner: postgres
--

CREATE TEXT SEARCH CONFIGURATION public.zh_cfg (
    PARSER = public.zhparser );

ALTER TEXT SEARCH CONFIGURATION public.zh_cfg
    ADD MAPPING FOR a WITH simple;

ALTER TEXT SEARCH CONFIGURATION public.zh_cfg
    ADD MAPPING FOR e WITH simple;

ALTER TEXT SEARCH CONFIGURATION public.zh_cfg
    ADD MAPPING FOR i WITH simple;

ALTER TEXT SEARCH CONFIGURATION public.zh_cfg
    ADD MAPPING FOR l WITH simple;

ALTER TEXT SEARCH CONFIGURATION public.zh_cfg
    ADD MAPPING FOR n WITH simple;

ALTER TEXT SEARCH CONFIGURATION public.zh_cfg
    ADD MAPPING FOR v WITH simple;


ALTER TEXT SEARCH CONFIGURATION public.zh_cfg OWNER TO postgres;

--
-- Name: zhparser_words; Type: TABLE; Schema: base; Owner: postgres
--

CREATE TABLE base.zhparser_words
(
    word text
);


ALTER TABLE base.zhparser_words
    OWNER TO postgres;

--
-- Name: TABLE zhparser_words; Type: COMMENT; Schema: base; Owner: postgres
--

COMMENT ON TABLE base.zhparser_words IS '自定义分词列表（主要是人名）';


--
-- Name: administrators; Type: TABLE; Schema: checking; Owner: postgres
--

CREATE TABLE checking.administrators
(
    job_id     text     NOT NULL,
    department text     NOT NULL,
    uid        integer  NOT NULL,
    role       smallint NOT NULL,
    name       text     NOT NULL
);


ALTER TABLE checking.administrators
    OWNER TO postgres;

--
-- Name: students; Type: TABLE; Schema: checking; Owner: postgres
--

CREATE TABLE checking.students
(
    student_id      text NOT NULL,
    uid             integer,
    name            text NOT NULL,
    audit_time      timestamp without time zone,
    college         text,
    major           text,
    identity_number text NOT NULL,
    audit_admin     text
);


ALTER TABLE checking.students
    OWNER TO postgres;

--
-- Name: approval_view; Type: VIEW; Schema: checking; Owner: postgres
--

CREATE VIEW checking.approval_view AS
SELECT s.student_id,
       s.uid,
       s.name,
       s.audit_time,
       (((a.name || ' ('::text) || a.job_id) || ')'::text) AS audit_admin,
       s.college,
       s.major,
       s.identity_number
FROM (checking.students s
         LEFT JOIN checking.administrators a ON ((a.job_id = s.audit_admin)));


ALTER TABLE checking.approval_view
    OWNER TO postgres;

--
-- Name: balance; Type: TABLE; Schema: dormitory; Owner: postgres
--

CREATE TABLE dormitory.balance
(
    room          integer NOT NULL,
    base_balance  real,
    elec_balance  real,
    total_balance real,
    ts            timestamp with time zone DEFAULT now()
);


ALTER TABLE dormitory.balance
    OWNER TO postgres;

--
-- Name: consumption; Type: TABLE; Schema: dormitory; Owner: postgres
--

CREATE TABLE dormitory.consumption
(
    ts     timestamp with time zone DEFAULT now() NOT NULL,
    room   integer                                NOT NULL,
    amount real                                   NOT NULL
);


ALTER TABLE dormitory.consumption
    OWNER TO postgres;

--
-- Name: rooms; Type: TABLE; Schema: dormitory; Owner: postgres
--

CREATE TABLE dormitory.rooms
(
    building smallint NOT NULL,
    layer    smallint NOT NULL,
    id       integer  NOT NULL
);


ALTER TABLE dormitory.rooms
    OWNER TO postgres;

--
-- Name: category; Type: TABLE; Schema: edu; Owner: postgres
--

CREATE TABLE edu.category
(
    code  smallint NOT NULL,
    title text     NOT NULL
);


ALTER TABLE edu.category
    OWNER TO postgres;

--
-- Name: TABLE category; Type: COMMENT; Schema: edu; Owner: postgres
--

COMMENT ON TABLE edu.category IS '课程大类';


--
-- Name: courses; Type: TABLE; Schema: edu; Owner: postgres
--

CREATE TABLE edu.courses
(
    course_id       text                        NOT NULL,
    dyn_class_id    text                        NOT NULL,
    title           text                        NOT NULL,
    weeks           integer                     NOT NULL,
    day             integer                     NOT NULL,
    time_index      smallint                    NOT NULL,
    place           text                        NOT NULL,
    teacher         text[] DEFAULT '{}'::text[] NOT NULL,
    hours           smallint                    NOT NULL,
    credit          real                        NOT NULL,
    preferred_class text[] DEFAULT '{}'::text[] NOT NULL,
    campus          text                        NOT NULL
);


ALTER TABLE edu.courses
    OWNER TO postgres;

--
-- Name: TABLE courses; Type: COMMENT; Schema: edu; Owner: postgres
--

COMMENT ON TABLE edu.courses IS '全校课程列表';


--
-- Name: major_plan; Type: TABLE; Schema: edu; Owner: postgres
--

CREATE TABLE edu.major_plan
(
    major           text     NOT NULL,
    year            smallint NOT NULL,
    course_category smallint NOT NULL,
    code            text     NOT NULL,
    title           text     NOT NULL,
    has_test        boolean  NOT NULL,
    credit          real     NOT NULL,
    theory_hour     smallint NOT NULL,
    practice_hour   smallint NOT NULL,
    department      text     NOT NULL,
    term            smallint
);


ALTER TABLE edu.major_plan
    OWNER TO postgres;

--
-- Name: majors; Type: TABLE; Schema: edu; Owner: postgres
--

CREATE TABLE edu.majors
(
    category    text                                      NOT NULL,
    code        text,
    title       text                                      NOT NULL,
    last_update timestamp without time zone DEFAULT now() NOT NULL
);


ALTER TABLE edu.majors
    OWNER TO postgres;

--
-- Name: events; Type: TABLE; Schema: events; Owner: postgres
--

CREATE TABLE events.events
(
    event_id      integer                                   NOT NULL,
    publisher_uid integer                                   NOT NULL,
    title         character varying(50)                     NOT NULL,
    description   text                                      NOT NULL,
    start_time    timestamp without time zone               NOT NULL,
    end_time      timestamp without time zone,
    create_time   timestamp without time zone DEFAULT now() NOT NULL,
    tags          character varying(50)[],
    deleted       boolean                     DEFAULT false NOT NULL,
    max_people    smallint,
    place         character varying(30)                     NOT NULL,
    image         character varying(50),
    attachments   integer[]
);


ALTER TABLE events.events
    OWNER TO postgres;

--
-- Name: TABLE events; Type: COMMENT; Schema: events; Owner: postgres
--

COMMENT ON TABLE events.events IS '活动列表';


--
-- Name: sc_events; Type: TABLE; Schema: events; Owner: postgres
--

CREATE TABLE events.sc_events
(
    activity_id integer                                                   NOT NULL,
    title       character varying(128)                                    NOT NULL,
    start_time  timestamp without time zone,
    sign_time   timestamp without time zone,
    end_time    timestamp without time zone,
    place       character varying(50),
    duration    character varying(20),
    manager     character varying(50),
    contact     character varying(50),
    organizer   character varying(50),
    undertaker  character varying(50),
    description text,
    hide        boolean                 DEFAULT false                     NOT NULL,
    tags        character varying(50)[] DEFAULT '{}'::character varying[] NOT NULL,
    image       text
);


ALTER TABLE events.sc_events
    OWNER TO postgres;

--
-- Name: TABLE sc_events; Type: COMMENT; Schema: events; Owner: postgres
--

COMMENT ON TABLE events.sc_events IS '第二课堂活动列表';


--
-- Name: person; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.person
(
    uid         integer                                   NOT NULL,
    nick_name   character varying(20)                     NOT NULL,
    avatar      text                                      NOT NULL,
    is_disabled boolean                     DEFAULT false NOT NULL,
    gender      smallint                    DEFAULT 0     NOT NULL,
    country     character varying(128),
    province    character varying(50),
    city        character varying(50),
    language    character varying(30),
    create_time timestamp without time zone DEFAULT now() NOT NULL,
    is_admin    boolean                     DEFAULT false
);


ALTER TABLE public.person
    OWNER TO postgres;

--
-- Name: all_events; Type: VIEW; Schema: events; Owner: postgres
--

CREATE VIEW events.all_events AS
SELECT 0                     AS source,
       sc_events.activity_id AS id,
       NULL::integer         AS publisher_uid,
       sc_events.manager     AS publisher_name,
       sc_events.title,
       sc_events.tags,
       sc_events.start_time,
       sc_events.end_time,
       sc_events.place,
       sc_events.image
FROM events.sc_events
WHERE (sc_events.hide = false)
UNION ALL
SELECT 1               AS source,
       events.event_id AS id,
       events.publisher_uid,
       p.nick_name     AS publisher_name,
       events.title,
       events.tags,
       events.start_time,
       events.end_time,
       events.place,
       events.image
FROM (events.events
         LEFT JOIN public.person p ON ((events.publisher_uid = p.uid)))
WHERE ((events.title)::text ~~ '%%'::text)
ORDER BY 7 DESC;


ALTER TABLE events.all_events
    OWNER TO postgres;

--
-- Name: event_applicants; Type: TABLE; Schema: events; Owner: postgres
--

CREATE TABLE events.event_applicants
(
    id         integer                     NOT NULL,
    uid        integer                     NOT NULL,
    event_id   integer                     NOT NULL,
    apply_time timestamp without time zone NOT NULL,
    sign_time  timestamp without time zone,
    sign_type  integer,
    finished   boolean DEFAULT false       NOT NULL
);


ALTER TABLE events.event_applicants
    OWNER TO postgres;

--
-- Name: events_event_id_seq; Type: SEQUENCE; Schema: events; Owner: postgres
--

CREATE SEQUENCE events.events_event_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE events.events_event_id_seq
    OWNER TO postgres;

--
-- Name: events_event_id_seq; Type: SEQUENCE OWNED BY; Schema: events; Owner: postgres
--

ALTER SEQUENCE events.events_event_id_seq OWNED BY events.events.publisher_uid;


--
-- Name: tags; Type: TABLE; Schema: events; Owner: postgres
--

CREATE TABLE events.tags
(
    id            integer               NOT NULL,
    keyword       character varying(50) NOT NULL,
    hide          boolean DEFAULT false NOT NULL,
    priority      integer DEFAULT 0     NOT NULL,
    default_image uuid[]
);


ALTER TABLE events.tags
    OWNER TO postgres;

--
-- Name: tags_id_seq; Type: SEQUENCE; Schema: events; Owner: postgres
--

CREATE SEQUENCE events.tags_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE events.tags_id_seq
    OWNER TO postgres;

--
-- Name: tags_id_seq; Type: SEQUENCE OWNED BY; Schema: events; Owner: postgres
--

ALTER SEQUENCE events.tags_id_seq OWNED BY events.tags.id;


--
-- Name: change_log; Type: TABLE; Schema: freshman; Owner: postgres
--

CREATE TABLE freshman.change_log
(
    student_id  text                                      NOT NULL,
    "timestamp" timestamp without time zone DEFAULT now() NOT NULL,
    uid         integer,
    contact     jsonb
);


ALTER TABLE freshman.change_log
    OWNER TO postgres;

--
-- Name: TABLE change_log; Type: COMMENT; Schema: freshman; Owner: postgres
--

COMMENT ON TABLE freshman.change_log IS '账户绑定及联系方式修改记录';


--
-- Name: share_log; Type: TABLE; Schema: freshman; Owner: postgres
--

CREATE TABLE freshman.share_log
(
    ts         timestamp without time zone DEFAULT now() NOT NULL,
    student_id text                                      NOT NULL
);


ALTER TABLE freshman.share_log
    OWNER TO postgres;

--
-- Name: students; Type: TABLE; Schema: freshman; Owner: postgres
--

CREATE TABLE freshman.students
(
    student_id     character varying(20) NOT NULL,
    uid            integer,
    ticket         character varying(20),
    name           character varying(20) NOT NULL,
    college        character varying(30) NOT NULL,
    major          character varying(70) NOT NULL,
    room           integer               NOT NULL,
    building       character varying(10) NOT NULL,
    bed            character varying(10) NOT NULL,
    class          character varying(10) NOT NULL,
    province       character varying(20),
    city           character varying(40),
    graduated_from character varying(128),
    postcode       integer,
    visible        boolean DEFAULT true  NOT NULL,
    contact        jsonb,
    last_seen      timestamp without time zone,
    campus         character varying(10) NOT NULL,
    counselor_name character varying(20) NOT NULL,
    counselor_tel  character varying(50) NOT NULL,
    secret         character varying(6)  NOT NULL,
    gender         character(1)          NOT NULL
);


ALTER TABLE freshman.students
    OWNER TO postgres;

--
-- Name: TABLE students; Type: COMMENT; Schema: freshman; Owner: postgres
--

COMMENT ON TABLE freshman.students IS '新生数据';


--
-- Name: comments; Type: TABLE; Schema: mall; Owner: postgres
--

CREATE TABLE mall.comments
(
    id        integer                                NOT NULL,
    goods_id  integer                                NOT NULL,
    publisher integer                                NOT NULL,
    content   text                                   NOT NULL,
    ts        timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE mall.comments
    OWNER TO postgres;

--
-- Name: TABLE comments; Type: COMMENT; Schema: mall; Owner: postgres
--

COMMENT ON TABLE mall.comments IS '商品评论和留言';


--
-- Name: comments_id_seq; Type: SEQUENCE; Schema: mall; Owner: postgres
--

CREATE SEQUENCE mall.comments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE mall.comments_id_seq
    OWNER TO postgres;

--
-- Name: comments_id_seq; Type: SEQUENCE OWNED BY; Schema: mall; Owner: postgres
--

ALTER SEQUENCE mall.comments_id_seq OWNED BY mall.comments.id;


--
-- Name: favorites; Type: TABLE; Schema: mall; Owner: postgres
--

CREATE TABLE mall.favorites
(
    person integer                                NOT NULL,
    goods  integer                                NOT NULL,
    ts     timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE mall.favorites
    OWNER TO postgres;

--
-- Name: TABLE favorites; Type: COMMENT; Schema: mall; Owner: postgres
--

COMMENT ON TABLE mall.favorites IS '用户收藏';


--
-- Name: goods_id_seq; Type: SEQUENCE; Schema: mall; Owner: postgres
--

CREATE SEQUENCE mall.goods_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE mall.goods_id_seq
    OWNER TO postgres;

--
-- Name: goods_id_seq; Type: SEQUENCE OWNED BY; Schema: mall; Owner: postgres
--

ALTER SEQUENCE mall.goods_id_seq OWNED BY mall.goods.id;


--
-- Name: share_log; Type: TABLE; Schema: mall; Owner: postgres
--

CREATE TABLE mall.share_log
(
    goods  integer                                NOT NULL,
    person integer                                NOT NULL,
    ts     timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE mall.share_log
    OWNER TO postgres;

--
-- Name: TABLE share_log; Type: COMMENT; Schema: mall; Owner: postgres
--

COMMENT ON TABLE mall.share_log IS '商品分享记录';


--
-- Name: sorts; Type: TABLE; Schema: mall; Owner: postgres
--

CREATE TABLE mall.sorts
(
    id       integer NOT NULL,
    title    text    NOT NULL,
    priority smallint
);


ALTER TABLE mall.sorts
    OWNER TO postgres;

--
-- Name: TABLE sorts; Type: COMMENT; Schema: mall; Owner: postgres
--

COMMENT ON TABLE mall.sorts IS '商品分类';


--
-- Name: sort_id_seq; Type: SEQUENCE; Schema: mall; Owner: postgres
--

CREATE SEQUENCE mall.sort_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE mall.sort_id_seq
    OWNER TO postgres;

--
-- Name: sort_id_seq; Type: SEQUENCE OWNED BY; Schema: mall; Owner: postgres
--

ALTER SEQUENCE mall.sort_id_seq OWNED BY mall.sorts.id;


--
-- Name: textbooks; Type: TABLE; Schema: mall; Owner: postgres
--

CREATE TABLE mall.textbooks
(
    id           integer NOT NULL,
    isbn         character varying(13),
    title        text    NOT NULL,
    sub_title    text,
    press        text    NOT NULL,
    author       text,
    translator   text,
    price        real,
    edition      text,
    edition_date text,
    page         integer,
    tag          text,
    self_edited  boolean
);


ALTER TABLE mall.textbooks
    OWNER TO postgres;

--
-- Name: TABLE textbooks; Type: COMMENT; Schema: mall; Owner: postgres
--

COMMENT ON TABLE mall.textbooks IS '新华书店教材订购目录';


--
-- Name: textbooks_1_id_seq; Type: SEQUENCE; Schema: mall; Owner: postgres
--

CREATE SEQUENCE mall.textbooks_1_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE mall.textbooks_1_id_seq
    OWNER TO postgres;

--
-- Name: textbooks_1_id_seq; Type: SEQUENCE OWNED BY; Schema: mall; Owner: postgres
--

ALTER SEQUENCE mall.textbooks_1_id_seq OWNED BY mall.textbooks.id;


--
-- Name: views; Type: TABLE; Schema: mall; Owner: postgres
--

CREATE TABLE mall.views
(
    person integer                                NOT NULL,
    goods  integer                                NOT NULL,
    ts     timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE mall.views
    OWNER TO postgres;

--
-- Name: wish; Type: TABLE; Schema: mall; Owner: postgres
--

CREATE TABLE mall.wish
(
    goods  integer NOT NULL,
    person integer NOT NULL,
    ts     timestamp with time zone DEFAULT now()
);


ALTER TABLE mall.wish
    OWNER TO postgres;

--
-- Name: TABLE wish; Type: COMMENT; Schema: mall; Owner: postgres
--

COMMENT ON TABLE mall.wish IS '"想要"商品的人';


--
-- Name: attachments; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.attachments
(
    id          uuid                        NOT NULL,
    name        text,
    path        text,
    uploader    integer                     NOT NULL,
    upload_time timestamp without time zone NOT NULL,
    is_deleted  boolean DEFAULT false       NOT NULL,
    size        integer                     NOT NULL,
    url         text                        NOT NULL
);


ALTER TABLE public.attachments
    OWNER TO postgres;

--
-- Name: attachments_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.attachments_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.attachments_id_seq
    OWNER TO postgres;

--
-- Name: authentication; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.authentication
(
    uid        integer               NOT NULL,
    login_type integer               NOT NULL,
    account    character varying(40) NOT NULL,
    credential character varying(40)
);


ALTER TABLE public.authentication
    OWNER TO postgres;

--
-- Name: authentication_log; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.authentication_log
(
    ts         timestamp without time zone DEFAULT now() NOT NULL,
    uid        integer                                   NOT NULL,
    account    text                                      NOT NULL,
    credential text
);


ALTER TABLE public.authentication_log
    OWNER TO postgres;

--
-- Name: pages; Type: TABLE; Schema: search; Owner: postgres
--

CREATE TABLE search.pages
(
    title        text,
    host         text,
    path         text,
    publish_date date,
    update_date  date,
    link_count   smallint,
    content      text,
    disable      boolean
);


ALTER TABLE search.pages
    OWNER TO postgres;

--
-- Name: TABLE pages; Type: COMMENT; Schema: search; Owner: postgres
--

COMMENT ON TABLE search.pages IS '爬取到的文章';


--
-- Name: available_pages; Type: VIEW; Schema: public; Owner: postgres
--

CREATE VIEW public.available_pages AS
SELECT pages.title,
       (pages.host || pages.path) AS uri,
       pages.publish_date,
       pages.content
FROM search.pages
WHERE (pages.disable = false);


ALTER TABLE public.available_pages
    OWNER TO postgres;

--
-- Name: identities; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.identities
(
    uid             integer               NOT NULL,
    student_id      character(10)         NOT NULL,
    oa_secret       character varying(32),
    oa_certified    boolean DEFAULT false NOT NULL,
    identity_number character varying(18),
    real_name       character varying(40) NOT NULL
);


ALTER TABLE public.identities
    OWNER TO postgres;

--
-- Name: identities_uid_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.identities_uid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.identities_uid_seq
    OWNER TO postgres;

--
-- Name: identities_uid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.identities_uid_seq OWNED BY public.identities.oa_certified;


--
-- Name: motto; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.motto
(
    id          integer            NOT NULL,
    source      character varying(50),
    content     text               NOT NULL,
    impressions integer  DEFAULT 0 NOT NULL,
    length      smallint DEFAULT 0 NOT NULL
);


ALTER TABLE public.motto
    OWNER TO postgres;

--
-- Name: motto_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.motto_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.motto_id_seq
    OWNER TO postgres;

--
-- Name: motto_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.motto_id_seq OWNED BY public.motto.id;


--
-- Name: notice; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.notice
(
    id           integer                                   NOT NULL,
    publish_time timestamp without time zone DEFAULT now() NOT NULL,
    title        text                                      NOT NULL,
    content      text,
    expired      boolean                     DEFAULT false NOT NULL,
    top          boolean                     DEFAULT false NOT NULL
);


ALTER TABLE public.notice
    OWNER TO postgres;

--
-- Name: notice_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.notice_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.notice_id_seq
    OWNER TO postgres;

--
-- Name: notice_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.notice_id_seq OWNED BY public.notice.id;


--
-- Name: person_uid_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.person_uid_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.person_uid_seq
    OWNER TO postgres;

--
-- Name: person_uid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.person_uid_seq OWNED BY public.person.nick_name;


--
-- Name: attachments; Type: TABLE; Schema: search; Owner: postgres
--

CREATE TABLE search.attachments
(
    id         integer NOT NULL,
    title      text,
    host       text,
    path       text,
    ext        text,
    size       integer,
    local_name text,
    checksum   character(32),
    referer    text
);


ALTER TABLE search.attachments
    OWNER TO postgres;

--
-- Name: TABLE attachments; Type: COMMENT; Schema: search; Owner: postgres
--

COMMENT ON TABLE search.attachments IS '附件列表';


--
-- Name: attachments_id_seq; Type: SEQUENCE; Schema: search; Owner: postgres
--

CREATE SEQUENCE search.attachments_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE search.attachments_id_seq
    OWNER TO postgres;

--
-- Name: attachments_id_seq; Type: SEQUENCE OWNED BY; Schema: search; Owner: postgres
--

ALTER SEQUENCE search.attachments_id_seq OWNED BY search.attachments.id;


--
-- Name: notices; Type: TABLE; Schema: search; Owner: postgres
--

CREATE TABLE search.notices
(
    url          text                     NOT NULL,
    title        text                     NOT NULL,
    publish_time timestamp with time zone NOT NULL,
    department   text                     NOT NULL,
    author       text,
    sort         text                     NOT NULL,
    content      text                     NOT NULL
);


ALTER TABLE search.notices
    OWNER TO postgres;

--
-- Name: TABLE notices; Type: COMMENT; Schema: search; Owner: postgres
--

COMMENT ON TABLE search.notices IS 'OA 公告';


--
-- Name: events event_id; Type: DEFAULT; Schema: events; Owner: postgres
--

ALTER TABLE ONLY events.events
    ALTER COLUMN event_id SET DEFAULT nextval('events.events_event_id_seq'::regclass);


--
-- Name: tags id; Type: DEFAULT; Schema: events; Owner: postgres
--

ALTER TABLE ONLY events.tags
    ALTER COLUMN id SET DEFAULT nextval('events.tags_id_seq'::regclass);


--
-- Name: comments id; Type: DEFAULT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.comments
    ALTER COLUMN id SET DEFAULT nextval('mall.comments_id_seq'::regclass);


--
-- Name: goods id; Type: DEFAULT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.goods
    ALTER COLUMN id SET DEFAULT nextval('mall.goods_id_seq'::regclass);


--
-- Name: sorts id; Type: DEFAULT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.sorts
    ALTER COLUMN id SET DEFAULT nextval('mall.sort_id_seq'::regclass);


--
-- Name: textbooks id; Type: DEFAULT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.textbooks
    ALTER COLUMN id SET DEFAULT nextval('mall.textbooks_1_id_seq'::regclass);


--
-- Name: motto id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.motto
    ALTER COLUMN id SET DEFAULT nextval('public.motto_id_seq'::regclass);


--
-- Name: notice id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.notice
    ALTER COLUMN id SET DEFAULT nextval('public.notice_id_seq'::regclass);


--
-- Name: person uid; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.person
    ALTER COLUMN uid SET DEFAULT nextval('public.person_uid_seq'::regclass);


--
-- Name: attachments id; Type: DEFAULT; Schema: search; Owner: postgres
--

ALTER TABLE ONLY search.attachments
    ALTER COLUMN id SET DEFAULT nextval('search.attachments_id_seq'::regclass);


--
-- Name: students approvals_pk; Type: CONSTRAINT; Schema: checking; Owner: postgres
--

ALTER TABLE ONLY checking.students
    ADD CONSTRAINT approvals_pk PRIMARY KEY (student_id);


--
-- Name: rooms rooms_pk; Type: CONSTRAINT; Schema: dormitory; Owner: postgres
--

ALTER TABLE ONLY dormitory.rooms
    ADD CONSTRAINT rooms_pk UNIQUE (id);


--
-- Name: event_applicants event_applicants_pk; Type: CONSTRAINT; Schema: events; Owner: postgres
--

ALTER TABLE ONLY events.event_applicants
    ADD CONSTRAINT event_applicants_pk PRIMARY KEY (id);


--
-- Name: events events_pk; Type: CONSTRAINT; Schema: events; Owner: postgres
--

ALTER TABLE ONLY events.events
    ADD CONSTRAINT events_pk PRIMARY KEY (event_id);


--
-- Name: events events_pk_2; Type: CONSTRAINT; Schema: events; Owner: postgres
--

ALTER TABLE ONLY events.events
    ADD CONSTRAINT events_pk_2 UNIQUE (event_id);


--
-- Name: sc_events sc_events_pk; Type: CONSTRAINT; Schema: events; Owner: postgres
--

ALTER TABLE ONLY events.sc_events
    ADD CONSTRAINT sc_events_pk PRIMARY KEY (activity_id);


--
-- Name: tags tags_pk; Type: CONSTRAINT; Schema: events; Owner: postgres
--

ALTER TABLE ONLY events.tags
    ADD CONSTRAINT tags_pk PRIMARY KEY (id);


--
-- Name: students students_pk; Type: CONSTRAINT; Schema: freshman; Owner: postgres
--

ALTER TABLE ONLY freshman.students
    ADD CONSTRAINT students_pk PRIMARY KEY (student_id);


--
-- Name: students students_pk_2; Type: CONSTRAINT; Schema: freshman; Owner: postgres
--

ALTER TABLE ONLY freshman.students
    ADD CONSTRAINT students_pk_2 UNIQUE (ticket);


--
-- Name: comments comments_pk; Type: CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.comments
    ADD CONSTRAINT comments_pk PRIMARY KEY (id);


--
-- Name: favorites favorite_pk; Type: CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.favorites
    ADD CONSTRAINT favorite_pk PRIMARY KEY (person, goods);


--
-- Name: goods goods_pk; Type: CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.goods
    ADD CONSTRAINT goods_pk PRIMARY KEY (id);


--
-- Name: share_log share_log_pk; Type: CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.share_log
    ADD CONSTRAINT share_log_pk PRIMARY KEY (goods, person);


--
-- Name: sorts sort_pk; Type: CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.sorts
    ADD CONSTRAINT sort_pk PRIMARY KEY (id);


--
-- Name: textbooks textbooks_pk; Type: CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.textbooks
    ADD CONSTRAINT textbooks_pk PRIMARY KEY (id);


--
-- Name: textbooks textbooks_pk_2; Type: CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.textbooks
    ADD CONSTRAINT textbooks_pk_2 UNIQUE (isbn);


--
-- Name: wish wish_pk; Type: CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.wish
    ADD CONSTRAINT wish_pk PRIMARY KEY (goods, person);


--
-- Name: attachments attachments_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.attachments
    ADD CONSTRAINT attachments_pk PRIMARY KEY (id);


--
-- Name: identities identities_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.identities
    ADD CONSTRAINT identities_pk PRIMARY KEY (uid);


--
-- Name: motto motto_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.motto
    ADD CONSTRAINT motto_pk PRIMARY KEY (id);


--
-- Name: authentication one_auth_per_type_user; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.authentication
    ADD CONSTRAINT one_auth_per_type_user UNIQUE (uid, login_type);


--
-- Name: person persons_pk; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.person
    ADD CONSTRAINT persons_pk PRIMARY KEY (uid);


--
-- Name: person persons_uid_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.person
    ADD CONSTRAINT persons_uid_key UNIQUE (uid);


--
-- Name: attachments attachments_pk; Type: CONSTRAINT; Schema: search; Owner: postgres
--

ALTER TABLE ONLY search.attachments
    ADD CONSTRAINT attachments_pk PRIMARY KEY (id);


--
-- Name: consumption_room_index; Type: INDEX; Schema: dormitory; Owner: postgres
--

CREATE INDEX consumption_room_index ON dormitory.consumption USING btree (room);


--
-- Name: idx_ts_brin; Type: INDEX; Schema: dormitory; Owner: postgres
--

CREATE INDEX idx_ts_brin ON dormitory.consumption USING brin (ts);


--
-- Name: rooms_building_layer_index; Type: INDEX; Schema: dormitory; Owner: postgres
--

CREATE INDEX rooms_building_layer_index ON dormitory.rooms USING btree (building, layer);


--
-- Name: major_plan_major_year_index; Type: INDEX; Schema: edu; Owner: postgres
--

CREATE INDEX major_plan_major_year_index ON edu.major_plan USING btree (major, year);


--
-- Name: events_event_id_uindex; Type: INDEX; Schema: events; Owner: postgres
--

CREATE UNIQUE INDEX events_event_id_uindex ON events.events USING btree (event_id);


--
-- Name: sc_events_start_time_index; Type: INDEX; Schema: events; Owner: postgres
--

CREATE INDEX sc_events_start_time_index ON events.sc_events USING btree (start_time DESC);


--
-- Name: sc_events_title_index; Type: INDEX; Schema: events; Owner: postgres
--

CREATE INDEX sc_events_title_index ON events.sc_events USING btree (title);


--
-- Name: tags_keyword_uindex; Type: INDEX; Schema: events; Owner: postgres
--

CREATE UNIQUE INDEX tags_keyword_uindex ON events.tags USING btree (keyword);


--
-- Name: students_studentid_ticket_name_index; Type: INDEX; Schema: freshman; Owner: postgres
--

CREATE INDEX students_studentid_ticket_name_index ON freshman.students USING btree (student_id, ticket, name);


--
-- Name: students_uid_index; Type: INDEX; Schema: freshman; Owner: postgres
--

CREATE INDEX students_uid_index ON freshman.students USING btree (uid);


--
-- Name: comments_goods_id_index; Type: INDEX; Schema: mall; Owner: postgres
--

CREATE INDEX comments_goods_id_index ON mall.comments USING btree (goods_id);


--
-- Name: goods_publish_time_index; Type: INDEX; Schema: mall; Owner: postgres
--

CREATE INDEX goods_publish_time_index ON mall.goods USING btree (publish_time DESC);


--
-- Name: textbooks_id_uindex; Type: INDEX; Schema: mall; Owner: postgres
--

CREATE UNIQUE INDEX textbooks_id_uindex ON mall.textbooks USING btree (id);


--
-- Name: attachments_1_id_uindex; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX attachments_1_id_uindex ON public.attachments USING btree (id);


--
-- Name: motto_id_uindex; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX motto_id_uindex ON public.motto USING btree (id);


--
-- Name: idx_page_title_content; Type: INDEX; Schema: search; Owner: postgres
--

CREATE INDEX idx_page_title_content ON search.notices USING gin (to_tsvector('public.zh_cfg'::regconfig, (title || content)));


--
-- Name: balance on_update_balance; Type: TRIGGER; Schema: dormitory; Owner: postgres
--

CREATE TRIGGER on_update_balance
    BEFORE INSERT OR UPDATE
    ON dormitory.balance
    FOR EACH ROW
EXECUTE FUNCTION public.calc_consumption();


--
-- Name: students on_change_contact; Type: TRIGGER; Schema: freshman; Owner: postgres
--

CREATE TRIGGER on_change_contact
    AFTER UPDATE
    ON freshman.students
    FOR EACH ROW
EXECUTE FUNCTION freshman.record_contact_change();


--
-- Name: authentication on_change_authentication; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER on_change_authentication
    AFTER DELETE OR UPDATE
    ON public.authentication
    FOR EACH ROW
EXECUTE FUNCTION public.record_authentication_change();


--
-- Name: students approvals_person_uid_fk; Type: FK CONSTRAINT; Schema: checking; Owner: postgres
--

ALTER TABLE ONLY checking.students
    ADD CONSTRAINT approvals_person_uid_fk FOREIGN KEY (uid) REFERENCES public.person (uid);


--
-- Name: balance balance_rooms_id_fk; Type: FK CONSTRAINT; Schema: dormitory; Owner: postgres
--

ALTER TABLE ONLY dormitory.balance
    ADD CONSTRAINT balance_rooms_id_fk FOREIGN KEY (room) REFERENCES dormitory.rooms (id);


--
-- Name: consumption consumption_rooms_id_fk; Type: FK CONSTRAINT; Schema: dormitory; Owner: postgres
--

ALTER TABLE ONLY dormitory.consumption
    ADD CONSTRAINT consumption_rooms_id_fk FOREIGN KEY (room) REFERENCES dormitory.rooms (id);


--
-- Name: event_applicants event_applicants_persons_uid_fk; Type: FK CONSTRAINT; Schema: events; Owner: postgres
--

ALTER TABLE ONLY events.event_applicants
    ADD CONSTRAINT event_applicants_persons_uid_fk FOREIGN KEY (uid) REFERENCES public.person (uid);


--
-- Name: events events_persons_uid_fk; Type: FK CONSTRAINT; Schema: events; Owner: postgres
--

ALTER TABLE ONLY events.events
    ADD CONSTRAINT events_persons_uid_fk FOREIGN KEY (publisher_uid) REFERENCES public.person (uid);


--
-- Name: comments comments_goods_id_fk; Type: FK CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.comments
    ADD CONSTRAINT comments_goods_id_fk FOREIGN KEY (goods_id) REFERENCES mall.goods (id);


--
-- Name: comments comments_person_uid_fk; Type: FK CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.comments
    ADD CONSTRAINT comments_person_uid_fk FOREIGN KEY (publisher) REFERENCES public.person (uid);


--
-- Name: goods goods_person_uid_fk; Type: FK CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.goods
    ADD CONSTRAINT goods_person_uid_fk FOREIGN KEY (publisher) REFERENCES public.person (uid);


--
-- Name: goods goods_sort_id_fk; Type: FK CONSTRAINT; Schema: mall; Owner: postgres
--

ALTER TABLE ONLY mall.goods
    ADD CONSTRAINT goods_sort_id_fk FOREIGN KEY (sort) REFERENCES mall.sorts (id);


--
-- Name: identities identities_persons_uid_fk; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.identities
    ADD CONSTRAINT identities_persons_uid_fk FOREIGN KEY (uid) REFERENCES public.person (uid) ON UPDATE RESTRICT;


--
-- Name: authentication verifications_persons_uid_fk; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.authentication
    ADD CONSTRAINT verifications_persons_uid_fk FOREIGN KEY (uid) REFERENCES public.person (uid) ON UPDATE RESTRICT ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

