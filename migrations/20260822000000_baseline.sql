-- Migration 0: the schema as the laravel app leaves it.
--
-- A pg_dump --schema-only of the production-shaped database, verbatim apart
-- from the edits noted below. It is a snapshot, not a design: nothing here has
-- been renamed, pruned or improved, because the rebuild has to run against this
-- exact schema before it is allowed to change it. Everything after this file is
-- a forward migration written normally.
--
-- Two things were stripped from the dump, both of which break `sqlx migrate`:
--
--   \restrict / \unrestrict  psql meta-commands, not SQL. pg_dump 17 emits them
--                            and the migrator sends the file straight to the
--                            server, which rejects them.
--
--   SELECT pg_catalog.set_config('search_path', '', false)
--                            empties the search path for the session. Every
--                            object below is public-qualified so the dump does
--                            not need it, but sqlx's own bookkeeping table is
--                            not qualified — with an empty search_path the
--                            migration applies and then recording it fails.
--
-- This file has two jobs, and which one applies depends on the database:
--
--   a fresh one    `lighthouse-migrate run` executes it and the schema exists.
--                  This is the local, CI and contributor-clone path, and the
--                  only reason the file is 3,000 lines rather than a note.
--
--   an existing    `lighthouse-migrate baseline` records it as applied without
--                  executing it. Production and any laravel-owned database take
--                  this path; `run` there fails on the first CREATE TABLE,
--                  correctly, because that schema does not need building.
--
-- Either way the version ends up in _sqlx_migrations, so from the next forward
-- migration onward both kinds of database are on the same footing.
--
-- Not made idempotent with IF NOT EXISTS, which would collapse the two paths
-- into one command. Postgres has no ADD CONSTRAINT IF NOT EXISTS and there are
-- 115 of them below, so each would need a DO block swallowing duplicate_object.
-- And IF NOT EXISTS compares nothing: it skips a table that exists with
-- entirely different columns, reporting a clean migration over a schema that
-- has drifted.
--
-- PostgreSQL database dump
--


-- Dumped from database version 17.10
-- Dumped by pg_dump version 17.10

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: activity_log; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.activity_log (
    id bigint NOT NULL,
    log_name character varying(255),
    description text NOT NULL,
    subject_type character varying(255),
    subject_id bigint,
    causer_type character varying(255),
    causer_id bigint,
    properties json,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    event character varying(255),
    batch_uuid uuid
);


--
-- Name: activity_log_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.activity_log_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: activity_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.activity_log_id_seq OWNED BY public.activity_log.id;


--
-- Name: cache; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.cache (
    key character varying(255) NOT NULL,
    value text NOT NULL,
    expiration integer NOT NULL
);


--
-- Name: cache_locks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.cache_locks (
    key character varying(255) NOT NULL,
    owner character varying(255) NOT NULL,
    expiration integer NOT NULL
);


--
-- Name: coupons; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.coupons (
    id bigint NOT NULL,
    stripe_coupon_id character varying(255) NOT NULL,
    stripe_promotion_code_id character varying(255) NOT NULL,
    code character varying(255) NOT NULL,
    discount_type character varying(10) NOT NULL,
    discount_value integer NOT NULL,
    duration character varying(20) NOT NULL,
    duration_months integer,
    max_redemptions integer,
    times_redeemed integer DEFAULT 0 NOT NULL,
    restricted_plans json,
    expires_at timestamp(0) without time zone,
    created_by bigint,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    bdt_discount_taka integer
);


--
-- Name: coupons_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.coupons_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: coupons_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.coupons_id_seq OWNED BY public.coupons.id;


--
-- Name: course_translations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.course_translations (
    id bigint NOT NULL,
    course_id bigint NOT NULL,
    locale character varying(5) NOT NULL,
    title character varying(255) NOT NULL,
    description text,
    meta_title character varying(255),
    meta_description text,
    meta_keywords character varying(255),
    og_title character varying(255),
    og_description text,
    og_image character varying(255),
    metadata json,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: course_translations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.course_translations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: course_translations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.course_translations_id_seq OWNED BY public.course_translations.id;


--
-- Name: courses; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.courses (
    id bigint NOT NULL,
    slug character varying(255) NOT NULL,
    thumbnail_url character varying(500),
    is_published boolean DEFAULT false NOT NULL,
    price integer,
    published_at timestamp(0) without time zone,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    first_lesson_slug character varying(255),
    lab_slug character varying(255),
    details_component character varying(255),
    required_tier character varying(255) DEFAULT 'voyage'::character varying NOT NULL
);


--
-- Name: courses_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.courses_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: courses_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.courses_id_seq OWNED BY public.courses.id;


--
-- Name: failed_jobs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.failed_jobs (
    id bigint NOT NULL,
    uuid character varying(255) NOT NULL,
    connection text NOT NULL,
    queue text NOT NULL,
    payload text NOT NULL,
    exception text NOT NULL,
    failed_at timestamp(0) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


--
-- Name: failed_jobs_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.failed_jobs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: failed_jobs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.failed_jobs_id_seq OWNED BY public.failed_jobs.id;


--
-- Name: gifts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.gifts (
    id bigint NOT NULL,
    uuid uuid NOT NULL,
    gifter_user_id bigint NOT NULL,
    recipient_user_id bigint,
    coupon_id bigint,
    plan character varying(255) NOT NULL,
    welcome_message text,
    stripe_checkout_session_id character varying(255),
    status character varying(20) DEFAULT 'pending_payment'::character varying NOT NULL,
    redeemed_at timestamp(0) without time zone,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: gifts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.gifts_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: gifts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.gifts_id_seq OWNED BY public.gifts.id;


--
-- Name: job_batches; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.job_batches (
    id character varying(255) NOT NULL,
    name character varying(255) NOT NULL,
    total_jobs integer NOT NULL,
    pending_jobs integer NOT NULL,
    failed_jobs integer NOT NULL,
    failed_job_ids text NOT NULL,
    options text,
    cancelled_at integer,
    created_at integer NOT NULL,
    finished_at integer
);


--
-- Name: jobs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.jobs (
    id bigint NOT NULL,
    queue character varying(255) NOT NULL,
    payload text NOT NULL,
    attempts smallint NOT NULL,
    reserved_at integer,
    available_at integer NOT NULL,
    created_at integer NOT NULL
);


--
-- Name: jobs_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.jobs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: jobs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.jobs_id_seq OWNED BY public.jobs.id;


--
-- Name: lab_submissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.lab_submissions (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    lab_id bigint NOT NULL,
    language character varying(20) NOT NULL,
    outcome character varying(20) NOT NULL,
    tests_total smallint DEFAULT '0'::smallint NOT NULL,
    tests_passed smallint DEFAULT '0'::smallint NOT NULL,
    test_results json,
    code_snapshot json,
    has_race_detector boolean DEFAULT false NOT NULL,
    execution_time_ms integer,
    raw_output text,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: lab_submissions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.lab_submissions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: lab_submissions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.lab_submissions_id_seq OWNED BY public.lab_submissions.id;


--
-- Name: labs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.labs (
    id bigint NOT NULL,
    slug character varying(255) NOT NULL,
    name character varying(255) NOT NULL,
    short_description text,
    docker_image character varying(255) NOT NULL,
    memory_mb integer DEFAULT 512 NOT NULL,
    cpus numeric(3,1) DEFAULT '1'::numeric NOT NULL,
    has_editor boolean DEFAULT false NOT NULL,
    content_repo character varying(255),
    content_branch character varying(255),
    is_published boolean DEFAULT false NOT NULL,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    starting_point character varying(100),
    tags json,
    test_files json,
    blueprint text,
    tier character varying(255) DEFAULT 'seeker'::character varying NOT NULL,
    markdown text,
    terminal_count smallint DEFAULT '1'::smallint NOT NULL,
    category character varying(255),
    editor_type character varying(255),
    languages json,
    editor_files json,
    run_commands json,
    primary_tag character varying(255),
    difficulty character varying(255)
);


--
-- Name: labs_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.labs_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: labs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.labs_id_seq OWNED BY public.labs.id;


--
-- Name: lesson_bookmarks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.lesson_bookmarks (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    lesson_id bigint NOT NULL,
    start_offset integer NOT NULL,
    end_offset integer NOT NULL,
    selected_text character varying(500) NOT NULL,
    xpath_start character varying(1000),
    xpath_end character varying(1000),
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: lesson_bookmarks_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.lesson_bookmarks_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: lesson_bookmarks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.lesson_bookmarks_id_seq OWNED BY public.lesson_bookmarks.id;


--
-- Name: lesson_completions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.lesson_completions (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    lesson_id bigint NOT NULL,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: lesson_completions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.lesson_completions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: lesson_completions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.lesson_completions_id_seq OWNED BY public.lesson_completions.id;


--
-- Name: lesson_translations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.lesson_translations (
    id bigint NOT NULL,
    lesson_id bigint NOT NULL,
    locale character varying(5) NOT NULL,
    title character varying(255) NOT NULL,
    description text,
    content text,
    meta_title character varying(255),
    meta_description text,
    meta_keywords character varying(255),
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: lesson_translations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.lesson_translations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: lesson_translations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.lesson_translations_id_seq OWNED BY public.lesson_translations.id;


--
-- Name: lessons; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.lessons (
    id bigint NOT NULL,
    course_id bigint NOT NULL,
    slug character varying(255) NOT NULL,
    chapter_id integer NOT NULL,
    sort_order integer NOT NULL,
    editor_version character varying(10) DEFAULT 'v1'::character varying NOT NULL,
    is_published boolean DEFAULT false NOT NULL,
    visibility_level integer DEFAULT 1 NOT NULL,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    published_at timestamp(0) without time zone,
    is_content_locked boolean DEFAULT false NOT NULL,
    lab_slug character varying(255)
);


--
-- Name: lessons_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.lessons_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: lessons_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.lessons_id_seq OWNED BY public.lessons.id;


--
-- Name: migrations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.migrations (
    id integer NOT NULL,
    migration character varying(255) NOT NULL,
    batch integer NOT NULL
);


--
-- Name: migrations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.migrations_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: migrations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.migrations_id_seq OWNED BY public.migrations.id;


--
-- Name: newsletter_opens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.newsletter_opens (
    id bigint NOT NULL,
    newsletter_id bigint NOT NULL,
    user_id bigint NOT NULL,
    ip_address character varying(45),
    user_agent character varying(255),
    opened_at timestamp(0) without time zone NOT NULL,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: newsletter_opens_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.newsletter_opens_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: newsletter_opens_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.newsletter_opens_id_seq OWNED BY public.newsletter_opens.id;


--
-- Name: newsletter_subscribers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.newsletter_subscribers (
    id bigint NOT NULL,
    email character varying(255) NOT NULL,
    language character varying(2) DEFAULT 'en'::character varying NOT NULL,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: newsletter_subscribers_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.newsletter_subscribers_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: newsletter_subscribers_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.newsletter_subscribers_id_seq OWNED BY public.newsletter_subscribers.id;


--
-- Name: newsletters; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.newsletters (
    id bigint NOT NULL,
    title character varying(255) NOT NULL,
    content text NOT NULL,
    language character varying(2) NOT NULL,
    status character varying(255) DEFAULT 'draft'::character varying NOT NULL,
    sent_at timestamp(0) without time zone,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: newsletters_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.newsletters_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: newsletters_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.newsletters_id_seq OWNED BY public.newsletters.id;


--
-- Name: notes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.notes (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    lesson_id bigint NOT NULL,
    selected_text text,
    note_content text,
    start_offset integer,
    end_offset integer,
    xpath_start character varying(255),
    xpath_end character varying(255),
    is_public boolean DEFAULT false NOT NULL,
    content_hash character varying(64),
    context_before text,
    context_after text,
    is_outdated boolean DEFAULT false NOT NULL,
    confidence_score integer,
    last_verified_at timestamp(0) without time zone,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    parent_id bigint
);


--
-- Name: notes_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.notes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: notes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.notes_id_seq OWNED BY public.notes.id;


--
-- Name: notifications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.notifications (
    id uuid NOT NULL,
    type character varying(255) NOT NULL,
    notifiable_type character varying(255) NOT NULL,
    notifiable_id bigint NOT NULL,
    data jsonb NOT NULL,
    read_at timestamp(0) without time zone,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: password_reset_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.password_reset_tokens (
    email character varying(255) NOT NULL,
    token character varying(255) NOT NULL,
    created_at timestamp(0) without time zone
);


--
-- Name: payment_requests; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.payment_requests (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    email character varying(100) NOT NULL,
    name character varying(100) NOT NULL,
    plan character varying(32) NOT NULL,
    amount_paisa bigint NOT NULL,
    transaction_id character varying(100) NOT NULL,
    beneficiary_name character varying(100) NOT NULL,
    note character varying(250),
    internal_note text,
    status smallint DEFAULT '0'::smallint NOT NULL,
    reviewed_by bigint,
    reviewed_at timestamp(0) without time zone,
    subscription_id bigint,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    coupon_code character varying(50)
);


--
-- Name: payment_requests_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.payment_requests_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: payment_requests_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.payment_requests_id_seq OWNED BY public.payment_requests.id;


--
-- Name: personal_access_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.personal_access_tokens (
    id bigint NOT NULL,
    tokenable_type character varying(255) NOT NULL,
    tokenable_id bigint NOT NULL,
    name text NOT NULL,
    token character varying(64) NOT NULL,
    abilities text,
    last_used_at timestamp(0) without time zone,
    expires_at timestamp(0) without time zone,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: personal_access_tokens_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.personal_access_tokens_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: personal_access_tokens_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.personal_access_tokens_id_seq OWNED BY public.personal_access_tokens.id;


--
-- Name: project_attempt_groups; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.project_attempt_groups (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    project_id bigint NOT NULL,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: project_attempt_groups_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.project_attempt_groups_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: project_attempt_groups_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.project_attempt_groups_id_seq OWNED BY public.project_attempt_groups.id;


--
-- Name: project_translations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.project_translations (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    locale character varying(5) NOT NULL,
    name character varying(255) NOT NULL,
    short_description character varying(255),
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: project_translations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.project_translations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: project_translations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.project_translations_id_seq OWNED BY public.project_translations.id;


--
-- Name: projects; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.projects (
    id bigint NOT NULL,
    slug character varying(255) NOT NULL,
    is_published boolean DEFAULT false NOT NULL,
    published_at timestamp(0) without time zone,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    metadata json,
    headline character varying(100),
    long_description text,
    features json,
    images json,
    is_featured boolean DEFAULT false NOT NULL,
    featured_order smallint DEFAULT '0'::smallint NOT NULL,
    show_tasks boolean DEFAULT true NOT NULL,
    runner_image character varying(255),
    uuid uuid,
    unlock_mode smallint DEFAULT '0'::smallint NOT NULL,
    markdown text,
    is_challenge boolean DEFAULT false NOT NULL,
    course_id bigint,
    course_sort_order integer,
    lab_slug character varying(255),
    difficulty character varying(255)
);


--
-- Name: projects_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.projects_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: projects_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.projects_id_seq OWNED BY public.projects.id;


--
-- Name: sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.sessions (
    id character varying(255) NOT NULL,
    user_id bigint,
    ip_address character varying(45),
    user_agent text,
    payload text NOT NULL,
    last_activity integer NOT NULL
);


--
-- Name: subscription_items; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.subscription_items (
    id bigint NOT NULL,
    subscription_id bigint NOT NULL,
    stripe_id character varying(255) NOT NULL,
    stripe_product character varying(255) NOT NULL,
    stripe_price character varying(255) NOT NULL,
    quantity integer,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    meter_id character varying(255),
    meter_event_name character varying(255)
);


--
-- Name: subscription_items_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.subscription_items_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: subscription_items_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.subscription_items_id_seq OWNED BY public.subscription_items.id;


--
-- Name: subscriptions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.subscriptions (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    type character varying(255) NOT NULL,
    stripe_id character varying(255) NOT NULL,
    stripe_status character varying(255) NOT NULL,
    stripe_price character varying(255),
    quantity integer,
    trial_ends_at timestamp(0) without time zone,
    ends_at timestamp(0) without time zone,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: subscriptions_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.subscriptions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: subscriptions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.subscriptions_id_seq OWNED BY public.subscriptions.id;


--
-- Name: task_hints; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.task_hints (
    id bigint NOT NULL,
    task_id bigint NOT NULL,
    text text NOT NULL,
    unlock_criteria character varying(50) NOT NULL,
    points_deduction integer DEFAULT 5 NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    uuid uuid
);


--
-- Name: task_hints_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.task_hints_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: task_hints_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.task_hints_id_seq OWNED BY public.task_hints.id;


--
-- Name: task_translations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.task_translations (
    id bigint NOT NULL,
    task_id bigint NOT NULL,
    locale character varying(5) NOT NULL,
    title character varying(255) NOT NULL,
    description text,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: task_translations_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.task_translations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: task_translations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.task_translations_id_seq OWNED BY public.task_translations.id;


--
-- Name: task_validators; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.task_validators (
    id bigint NOT NULL,
    task_id bigint NOT NULL,
    validator_dsl character varying(1000) NOT NULL,
    sort_order integer DEFAULT 0 NOT NULL,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: task_validators_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.task_validators_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: task_validators_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.task_validators_id_seq OWNED BY public.task_validators.id;


--
-- Name: tasks; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tasks (
    id bigint NOT NULL,
    project_id bigint NOT NULL,
    sort_order integer DEFAULT 1 NOT NULL,
    visibility_level integer DEFAULT 1 NOT NULL,
    points integer DEFAULT 0 NOT NULL,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    metadata json,
    slug character varying(255) NOT NULL,
    scores character varying(255),
    abandoned_deduction integer DEFAULT 5 NOT NULL,
    uuid uuid,
    input_type smallint DEFAULT '0'::smallint NOT NULL,
    is_free boolean DEFAULT false NOT NULL,
    blueprint text
);


--
-- Name: tasks_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.tasks_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: tasks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.tasks_id_seq OWNED BY public.tasks.id;


--
-- Name: user_lab_ssh_keys; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_lab_ssh_keys (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    name character varying(255) NOT NULL,
    public_key text NOT NULL,
    private_key text NOT NULL,
    fingerprint character varying(255) NOT NULL,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: user_lab_ssh_keys_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_lab_ssh_keys_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_lab_ssh_keys_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_lab_ssh_keys_id_seq OWNED BY public.user_lab_ssh_keys.id;


--
-- Name: user_project_attempts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_project_attempts (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    project_id bigint NOT NULL,
    task_id bigint NOT NULL,
    task_outcome character varying(20) DEFAULT 'attempted'::character varying NOT NULL,
    points_achieved integer DEFAULT 0 NOT NULL,
    task_outcome_context text,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    is_reattempt boolean DEFAULT false NOT NULL,
    attempt_group_id bigint
);


--
-- Name: user_project_attempts_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_project_attempts_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_project_attempts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_project_attempts_id_seq OWNED BY public.user_project_attempts.id;


--
-- Name: user_task_progress; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_task_progress (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    task_id bigint NOT NULL,
    status character varying(50) DEFAULT 'challenge_awaits'::character varying NOT NULL,
    attempts integer DEFAULT 0 NOT NULL,
    started_at timestamp(0) without time zone,
    completed_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    attempt_group_id bigint
);


--
-- Name: user_task_progress_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_task_progress_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_task_progress_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_task_progress_id_seq OWNED BY public.user_task_progress.id;


--
-- Name: user_unlocked_hints; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.user_unlocked_hints (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    task_id bigint NOT NULL,
    task_hint_id bigint NOT NULL,
    points_deducted integer DEFAULT 0 NOT NULL,
    unlocked_at timestamp(0) without time zone NOT NULL
);


--
-- Name: user_unlocked_hints_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.user_unlocked_hints_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: user_unlocked_hints_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.user_unlocked_hints_id_seq OWNED BY public.user_unlocked_hints.id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.users (
    id bigint NOT NULL,
    name character varying(255) NOT NULL,
    email character varying(255) NOT NULL,
    email_verified_at timestamp(0) without time zone,
    password character varying(255),
    remember_token character varying(100),
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone,
    two_factor_secret text,
    two_factor_recovery_codes text,
    two_factor_confirmed_at timestamp(0) without time zone,
    avatar_url character varying(500),
    google_id character varying(255),
    github_id character varying(255),
    stripe_id character varying(255),
    pm_type character varying(255),
    pm_last_four character varying(4),
    trial_ends_at timestamp(0) without time zone,
    github_username character varying(255),
    linkedin_url character varying(255),
    bio text,
    company character varying(255),
    education character varying(255),
    newsletter_enabled boolean DEFAULT false NOT NULL,
    newsletter_language character varying(2),
    username character varying(255),
    tagline character varying(160),
    location character varying(120),
    x_url character varying(255),
    website_url character varying(255)
);


--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.users_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: votes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.votes (
    id bigint NOT NULL,
    user_id bigint NOT NULL,
    subject_key character varying(100) NOT NULL,
    value smallint NOT NULL,
    comment text,
    created_at timestamp(0) without time zone,
    updated_at timestamp(0) without time zone
);


--
-- Name: votes_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.votes_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: votes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.votes_id_seq OWNED BY public.votes.id;


--
-- Name: activity_log id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.activity_log ALTER COLUMN id SET DEFAULT nextval('public.activity_log_id_seq'::regclass);


--
-- Name: coupons id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.coupons ALTER COLUMN id SET DEFAULT nextval('public.coupons_id_seq'::regclass);


--
-- Name: course_translations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.course_translations ALTER COLUMN id SET DEFAULT nextval('public.course_translations_id_seq'::regclass);


--
-- Name: courses id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.courses ALTER COLUMN id SET DEFAULT nextval('public.courses_id_seq'::regclass);


--
-- Name: failed_jobs id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.failed_jobs ALTER COLUMN id SET DEFAULT nextval('public.failed_jobs_id_seq'::regclass);


--
-- Name: gifts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gifts ALTER COLUMN id SET DEFAULT nextval('public.gifts_id_seq'::regclass);


--
-- Name: jobs id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.jobs ALTER COLUMN id SET DEFAULT nextval('public.jobs_id_seq'::regclass);


--
-- Name: lab_submissions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lab_submissions ALTER COLUMN id SET DEFAULT nextval('public.lab_submissions_id_seq'::regclass);


--
-- Name: labs id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.labs ALTER COLUMN id SET DEFAULT nextval('public.labs_id_seq'::regclass);


--
-- Name: lesson_bookmarks id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_bookmarks ALTER COLUMN id SET DEFAULT nextval('public.lesson_bookmarks_id_seq'::regclass);


--
-- Name: lesson_completions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_completions ALTER COLUMN id SET DEFAULT nextval('public.lesson_completions_id_seq'::regclass);


--
-- Name: lesson_translations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_translations ALTER COLUMN id SET DEFAULT nextval('public.lesson_translations_id_seq'::regclass);


--
-- Name: lessons id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lessons ALTER COLUMN id SET DEFAULT nextval('public.lessons_id_seq'::regclass);


--
-- Name: migrations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.migrations ALTER COLUMN id SET DEFAULT nextval('public.migrations_id_seq'::regclass);


--
-- Name: newsletter_opens id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.newsletter_opens ALTER COLUMN id SET DEFAULT nextval('public.newsletter_opens_id_seq'::regclass);


--
-- Name: newsletter_subscribers id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.newsletter_subscribers ALTER COLUMN id SET DEFAULT nextval('public.newsletter_subscribers_id_seq'::regclass);


--
-- Name: newsletters id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.newsletters ALTER COLUMN id SET DEFAULT nextval('public.newsletters_id_seq'::regclass);


--
-- Name: notes id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notes ALTER COLUMN id SET DEFAULT nextval('public.notes_id_seq'::regclass);


--
-- Name: payment_requests id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_requests ALTER COLUMN id SET DEFAULT nextval('public.payment_requests_id_seq'::regclass);


--
-- Name: personal_access_tokens id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.personal_access_tokens ALTER COLUMN id SET DEFAULT nextval('public.personal_access_tokens_id_seq'::regclass);


--
-- Name: project_attempt_groups id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_attempt_groups ALTER COLUMN id SET DEFAULT nextval('public.project_attempt_groups_id_seq'::regclass);


--
-- Name: project_translations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_translations ALTER COLUMN id SET DEFAULT nextval('public.project_translations_id_seq'::regclass);


--
-- Name: projects id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.projects ALTER COLUMN id SET DEFAULT nextval('public.projects_id_seq'::regclass);


--
-- Name: subscription_items id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_items ALTER COLUMN id SET DEFAULT nextval('public.subscription_items_id_seq'::regclass);


--
-- Name: subscriptions id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscriptions ALTER COLUMN id SET DEFAULT nextval('public.subscriptions_id_seq'::regclass);


--
-- Name: task_hints id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_hints ALTER COLUMN id SET DEFAULT nextval('public.task_hints_id_seq'::regclass);


--
-- Name: task_translations id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_translations ALTER COLUMN id SET DEFAULT nextval('public.task_translations_id_seq'::regclass);


--
-- Name: task_validators id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_validators ALTER COLUMN id SET DEFAULT nextval('public.task_validators_id_seq'::regclass);


--
-- Name: tasks id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tasks ALTER COLUMN id SET DEFAULT nextval('public.tasks_id_seq'::regclass);


--
-- Name: user_lab_ssh_keys id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_lab_ssh_keys ALTER COLUMN id SET DEFAULT nextval('public.user_lab_ssh_keys_id_seq'::regclass);


--
-- Name: user_project_attempts id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_project_attempts ALTER COLUMN id SET DEFAULT nextval('public.user_project_attempts_id_seq'::regclass);


--
-- Name: user_task_progress id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_task_progress ALTER COLUMN id SET DEFAULT nextval('public.user_task_progress_id_seq'::regclass);


--
-- Name: user_unlocked_hints id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_unlocked_hints ALTER COLUMN id SET DEFAULT nextval('public.user_unlocked_hints_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: votes id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.votes ALTER COLUMN id SET DEFAULT nextval('public.votes_id_seq'::regclass);


--
-- Name: activity_log activity_log_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.activity_log
    ADD CONSTRAINT activity_log_pkey PRIMARY KEY (id);


--
-- Name: cache_locks cache_locks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cache_locks
    ADD CONSTRAINT cache_locks_pkey PRIMARY KEY (key);


--
-- Name: cache cache_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.cache
    ADD CONSTRAINT cache_pkey PRIMARY KEY (key);


--
-- Name: coupons coupons_code_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.coupons
    ADD CONSTRAINT coupons_code_unique UNIQUE (code);


--
-- Name: coupons coupons_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.coupons
    ADD CONSTRAINT coupons_pkey PRIMARY KEY (id);


--
-- Name: coupons coupons_stripe_coupon_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.coupons
    ADD CONSTRAINT coupons_stripe_coupon_id_unique UNIQUE (stripe_coupon_id);


--
-- Name: coupons coupons_stripe_promotion_code_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.coupons
    ADD CONSTRAINT coupons_stripe_promotion_code_id_unique UNIQUE (stripe_promotion_code_id);


--
-- Name: course_translations course_translations_course_id_locale_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.course_translations
    ADD CONSTRAINT course_translations_course_id_locale_unique UNIQUE (course_id, locale);


--
-- Name: course_translations course_translations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.course_translations
    ADD CONSTRAINT course_translations_pkey PRIMARY KEY (id);


--
-- Name: courses courses_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.courses
    ADD CONSTRAINT courses_pkey PRIMARY KEY (id);


--
-- Name: courses courses_slug_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.courses
    ADD CONSTRAINT courses_slug_unique UNIQUE (slug);


--
-- Name: failed_jobs failed_jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.failed_jobs
    ADD CONSTRAINT failed_jobs_pkey PRIMARY KEY (id);


--
-- Name: failed_jobs failed_jobs_uuid_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.failed_jobs
    ADD CONSTRAINT failed_jobs_uuid_unique UNIQUE (uuid);


--
-- Name: gifts gifts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gifts
    ADD CONSTRAINT gifts_pkey PRIMARY KEY (id);


--
-- Name: gifts gifts_uuid_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gifts
    ADD CONSTRAINT gifts_uuid_unique UNIQUE (uuid);


--
-- Name: job_batches job_batches_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.job_batches
    ADD CONSTRAINT job_batches_pkey PRIMARY KEY (id);


--
-- Name: jobs jobs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.jobs
    ADD CONSTRAINT jobs_pkey PRIMARY KEY (id);


--
-- Name: lab_submissions lab_submissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lab_submissions
    ADD CONSTRAINT lab_submissions_pkey PRIMARY KEY (id);


--
-- Name: labs labs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.labs
    ADD CONSTRAINT labs_pkey PRIMARY KEY (id);


--
-- Name: labs labs_slug_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.labs
    ADD CONSTRAINT labs_slug_unique UNIQUE (slug);


--
-- Name: lesson_bookmarks lesson_bookmarks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_bookmarks
    ADD CONSTRAINT lesson_bookmarks_pkey PRIMARY KEY (id);


--
-- Name: lesson_bookmarks lesson_bookmarks_user_id_lesson_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_bookmarks
    ADD CONSTRAINT lesson_bookmarks_user_id_lesson_id_unique UNIQUE (user_id, lesson_id);


--
-- Name: lesson_completions lesson_completions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_completions
    ADD CONSTRAINT lesson_completions_pkey PRIMARY KEY (id);


--
-- Name: lesson_completions lesson_completions_user_id_lesson_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_completions
    ADD CONSTRAINT lesson_completions_user_id_lesson_id_unique UNIQUE (user_id, lesson_id);


--
-- Name: lesson_translations lesson_translations_lesson_id_locale_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_translations
    ADD CONSTRAINT lesson_translations_lesson_id_locale_unique UNIQUE (lesson_id, locale);


--
-- Name: lesson_translations lesson_translations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_translations
    ADD CONSTRAINT lesson_translations_pkey PRIMARY KEY (id);


--
-- Name: lessons lessons_course_id_slug_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lessons
    ADD CONSTRAINT lessons_course_id_slug_unique UNIQUE (course_id, slug);


--
-- Name: lessons lessons_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lessons
    ADD CONSTRAINT lessons_pkey PRIMARY KEY (id);


--
-- Name: migrations migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.migrations
    ADD CONSTRAINT migrations_pkey PRIMARY KEY (id);


--
-- Name: newsletter_opens newsletter_opens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.newsletter_opens
    ADD CONSTRAINT newsletter_opens_pkey PRIMARY KEY (id);


--
-- Name: newsletter_subscribers newsletter_subscribers_email_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.newsletter_subscribers
    ADD CONSTRAINT newsletter_subscribers_email_unique UNIQUE (email);


--
-- Name: newsletter_subscribers newsletter_subscribers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.newsletter_subscribers
    ADD CONSTRAINT newsletter_subscribers_pkey PRIMARY KEY (id);


--
-- Name: newsletters newsletters_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.newsletters
    ADD CONSTRAINT newsletters_pkey PRIMARY KEY (id);


--
-- Name: notes notes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notes
    ADD CONSTRAINT notes_pkey PRIMARY KEY (id);


--
-- Name: notifications notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_pkey PRIMARY KEY (id);


--
-- Name: password_reset_tokens password_reset_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_pkey PRIMARY KEY (email);


--
-- Name: payment_requests payment_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_requests
    ADD CONSTRAINT payment_requests_pkey PRIMARY KEY (id);


--
-- Name: payment_requests payment_requests_transaction_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_requests
    ADD CONSTRAINT payment_requests_transaction_id_unique UNIQUE (transaction_id);


--
-- Name: personal_access_tokens personal_access_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.personal_access_tokens
    ADD CONSTRAINT personal_access_tokens_pkey PRIMARY KEY (id);


--
-- Name: personal_access_tokens personal_access_tokens_token_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.personal_access_tokens
    ADD CONSTRAINT personal_access_tokens_token_unique UNIQUE (token);


--
-- Name: project_attempt_groups project_attempt_groups_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_attempt_groups
    ADD CONSTRAINT project_attempt_groups_pkey PRIMARY KEY (id);


--
-- Name: project_translations project_translations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_translations
    ADD CONSTRAINT project_translations_pkey PRIMARY KEY (id);


--
-- Name: project_translations project_translations_project_id_locale_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_translations
    ADD CONSTRAINT project_translations_project_id_locale_unique UNIQUE (project_id, locale);


--
-- Name: projects projects_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_pkey PRIMARY KEY (id);


--
-- Name: projects projects_slug_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_slug_unique UNIQUE (slug);


--
-- Name: projects projects_uuid_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_uuid_unique UNIQUE (uuid);


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);


--
-- Name: subscription_items subscription_items_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_items
    ADD CONSTRAINT subscription_items_pkey PRIMARY KEY (id);


--
-- Name: subscription_items subscription_items_stripe_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_items
    ADD CONSTRAINT subscription_items_stripe_id_unique UNIQUE (stripe_id);


--
-- Name: subscription_items subscription_items_subscription_id_stripe_price_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_items
    ADD CONSTRAINT subscription_items_subscription_id_stripe_price_unique UNIQUE (subscription_id, stripe_price);


--
-- Name: subscriptions subscriptions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscriptions
    ADD CONSTRAINT subscriptions_pkey PRIMARY KEY (id);


--
-- Name: subscriptions subscriptions_stripe_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscriptions
    ADD CONSTRAINT subscriptions_stripe_id_unique UNIQUE (stripe_id);


--
-- Name: task_hints task_hints_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_hints
    ADD CONSTRAINT task_hints_pkey PRIMARY KEY (id);


--
-- Name: task_hints task_hints_uuid_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_hints
    ADD CONSTRAINT task_hints_uuid_unique UNIQUE (uuid);


--
-- Name: task_translations task_translations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_translations
    ADD CONSTRAINT task_translations_pkey PRIMARY KEY (id);


--
-- Name: task_translations task_translations_task_id_locale_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_translations
    ADD CONSTRAINT task_translations_task_id_locale_unique UNIQUE (task_id, locale);


--
-- Name: task_validators task_validators_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_validators
    ADD CONSTRAINT task_validators_pkey PRIMARY KEY (id);


--
-- Name: tasks tasks_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tasks
    ADD CONSTRAINT tasks_pkey PRIMARY KEY (id);


--
-- Name: tasks tasks_project_id_slug_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tasks
    ADD CONSTRAINT tasks_project_id_slug_unique UNIQUE (project_id, slug);


--
-- Name: tasks tasks_project_id_sort_order_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tasks
    ADD CONSTRAINT tasks_project_id_sort_order_unique UNIQUE (project_id, sort_order);


--
-- Name: tasks tasks_uuid_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tasks
    ADD CONSTRAINT tasks_uuid_unique UNIQUE (uuid);


--
-- Name: user_lab_ssh_keys user_lab_ssh_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_lab_ssh_keys
    ADD CONSTRAINT user_lab_ssh_keys_pkey PRIMARY KEY (id);


--
-- Name: user_project_attempts user_project_attempts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_project_attempts
    ADD CONSTRAINT user_project_attempts_pkey PRIMARY KEY (id);


--
-- Name: user_task_progress user_task_progress_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_task_progress
    ADD CONSTRAINT user_task_progress_pkey PRIMARY KEY (id);


--
-- Name: user_task_progress user_task_progress_user_id_task_id_attempt_group_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_task_progress
    ADD CONSTRAINT user_task_progress_user_id_task_id_attempt_group_id_unique UNIQUE (user_id, task_id, attempt_group_id);


--
-- Name: user_unlocked_hints user_unlocked_hints_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_unlocked_hints
    ADD CONSTRAINT user_unlocked_hints_pkey PRIMARY KEY (id);


--
-- Name: user_unlocked_hints user_unlocked_hints_user_id_task_hint_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_unlocked_hints
    ADD CONSTRAINT user_unlocked_hints_user_id_task_hint_id_unique UNIQUE (user_id, task_hint_id);


--
-- Name: users users_email_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_unique UNIQUE (email);


--
-- Name: users users_github_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_github_id_unique UNIQUE (github_id);


--
-- Name: users users_google_id_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_google_id_unique UNIQUE (google_id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_username_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_unique UNIQUE (username);


--
-- Name: votes votes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.votes
    ADD CONSTRAINT votes_pkey PRIMARY KEY (id);


--
-- Name: votes votes_user_id_subject_key_unique; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.votes
    ADD CONSTRAINT votes_user_id_subject_key_unique UNIQUE (user_id, subject_key);


--
-- Name: activity_log_log_name_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX activity_log_log_name_index ON public.activity_log USING btree (log_name);


--
-- Name: causer; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX causer ON public.activity_log USING btree (causer_type, causer_id);


--
-- Name: course_translations_locale_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX course_translations_locale_index ON public.course_translations USING btree (locale);


--
-- Name: courses_is_published_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX courses_is_published_index ON public.courses USING btree (is_published);


--
-- Name: courses_published_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX courses_published_at_index ON public.courses USING btree (published_at);


--
-- Name: courses_slug_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX courses_slug_index ON public.courses USING btree (slug);


--
-- Name: jobs_queue_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX jobs_queue_index ON public.jobs USING btree (queue);


--
-- Name: lab_submissions_lab_id_outcome_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lab_submissions_lab_id_outcome_index ON public.lab_submissions USING btree (lab_id, outcome);


--
-- Name: lab_submissions_user_id_lab_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lab_submissions_user_id_lab_id_index ON public.lab_submissions USING btree (user_id, lab_id);


--
-- Name: lab_submissions_user_id_lab_id_outcome_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lab_submissions_user_id_lab_id_outcome_index ON public.lab_submissions USING btree (user_id, lab_id, outcome);


--
-- Name: lesson_completions_created_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lesson_completions_created_at_index ON public.lesson_completions USING btree (created_at);


--
-- Name: lesson_completions_lesson_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lesson_completions_lesson_id_index ON public.lesson_completions USING btree (lesson_id);


--
-- Name: lesson_translations_locale_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lesson_translations_locale_index ON public.lesson_translations USING btree (locale);


--
-- Name: lessons_course_id_chapter_id_sort_order_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lessons_course_id_chapter_id_sort_order_index ON public.lessons USING btree (course_id, chapter_id, sort_order);


--
-- Name: lessons_course_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lessons_course_id_index ON public.lessons USING btree (course_id);


--
-- Name: lessons_course_id_slug_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lessons_course_id_slug_index ON public.lessons USING btree (course_id, slug);


--
-- Name: lessons_is_published_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lessons_is_published_index ON public.lessons USING btree (is_published);


--
-- Name: lessons_is_published_published_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lessons_is_published_published_at_index ON public.lessons USING btree (is_published, published_at);


--
-- Name: lessons_published_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX lessons_published_at_index ON public.lessons USING btree (published_at);


--
-- Name: newsletter_opens_newsletter_id_user_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX newsletter_opens_newsletter_id_user_id_index ON public.newsletter_opens USING btree (newsletter_id, user_id);


--
-- Name: newsletter_opens_opened_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX newsletter_opens_opened_at_index ON public.newsletter_opens USING btree (opened_at);


--
-- Name: newsletter_subscribers_email_language_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX newsletter_subscribers_email_language_index ON public.newsletter_subscribers USING btree (email, language);


--
-- Name: newsletters_sent_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX newsletters_sent_at_index ON public.newsletters USING btree (sent_at);


--
-- Name: newsletters_status_language_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX newsletters_status_language_index ON public.newsletters USING btree (status, language);


--
-- Name: notes_created_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX notes_created_at_index ON public.notes USING btree (created_at);


--
-- Name: notes_last_verified_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX notes_last_verified_at_index ON public.notes USING btree (last_verified_at);


--
-- Name: notes_lesson_id_is_outdated_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX notes_lesson_id_is_outdated_index ON public.notes USING btree (lesson_id, is_outdated);


--
-- Name: notes_lesson_id_is_public_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX notes_lesson_id_is_public_index ON public.notes USING btree (lesson_id, is_public);


--
-- Name: notes_parent_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX notes_parent_id_index ON public.notes USING btree (parent_id);


--
-- Name: notes_user_id_lesson_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX notes_user_id_lesson_id_index ON public.notes USING btree (user_id, lesson_id);


--
-- Name: notifications_notifiable_type_notifiable_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX notifications_notifiable_type_notifiable_id_index ON public.notifications USING btree (notifiable_type, notifiable_id);


--
-- Name: payment_requests_status_created_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX payment_requests_status_created_at_index ON public.payment_requests USING btree (status, created_at);


--
-- Name: payment_requests_user_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX payment_requests_user_id_index ON public.payment_requests USING btree (user_id);


--
-- Name: personal_access_tokens_expires_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX personal_access_tokens_expires_at_index ON public.personal_access_tokens USING btree (expires_at);


--
-- Name: personal_access_tokens_tokenable_type_tokenable_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX personal_access_tokens_tokenable_type_tokenable_id_index ON public.personal_access_tokens USING btree (tokenable_type, tokenable_id);


--
-- Name: project_attempt_groups_user_id_project_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX project_attempt_groups_user_id_project_id_index ON public.project_attempt_groups USING btree (user_id, project_id);


--
-- Name: project_translations_locale_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX project_translations_locale_index ON public.project_translations USING btree (locale);


--
-- Name: projects_course_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX projects_course_id_index ON public.projects USING btree (course_id);


--
-- Name: projects_featured_order_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX projects_featured_order_index ON public.projects USING btree (featured_order);


--
-- Name: projects_is_featured_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX projects_is_featured_index ON public.projects USING btree (is_featured);


--
-- Name: projects_is_published_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX projects_is_published_index ON public.projects USING btree (is_published);


--
-- Name: projects_published_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX projects_published_at_index ON public.projects USING btree (published_at);


--
-- Name: projects_slug_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX projects_slug_index ON public.projects USING btree (slug);


--
-- Name: sessions_last_activity_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX sessions_last_activity_index ON public.sessions USING btree (last_activity);


--
-- Name: sessions_user_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX sessions_user_id_index ON public.sessions USING btree (user_id);


--
-- Name: subject; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX subject ON public.activity_log USING btree (subject_type, subject_id);


--
-- Name: subscriptions_user_id_stripe_status_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX subscriptions_user_id_stripe_status_index ON public.subscriptions USING btree (user_id, stripe_status);


--
-- Name: task_hints_task_id_sort_order_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX task_hints_task_id_sort_order_index ON public.task_hints USING btree (task_id, sort_order);


--
-- Name: task_translations_locale_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX task_translations_locale_index ON public.task_translations USING btree (locale);


--
-- Name: task_validators_task_id_sort_order_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX task_validators_task_id_sort_order_index ON public.task_validators USING btree (task_id, sort_order);


--
-- Name: tasks_project_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX tasks_project_id_index ON public.tasks USING btree (project_id);


--
-- Name: tasks_project_id_sort_order_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX tasks_project_id_sort_order_index ON public.tasks USING btree (project_id, sort_order);


--
-- Name: user_lab_ssh_keys_user_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_lab_ssh_keys_user_id_index ON public.user_lab_ssh_keys USING btree (user_id);


--
-- Name: user_project_attempts_project_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_project_attempts_project_id_index ON public.user_project_attempts USING btree (project_id);


--
-- Name: user_project_attempts_task_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_project_attempts_task_id_index ON public.user_project_attempts USING btree (task_id);


--
-- Name: user_project_attempts_user_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_project_attempts_user_id_index ON public.user_project_attempts USING btree (user_id);


--
-- Name: user_project_attempts_user_id_project_id_attempt_group_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_project_attempts_user_id_project_id_attempt_group_id_index ON public.user_project_attempts USING btree (user_id, project_id, attempt_group_id);


--
-- Name: user_project_attempts_user_id_project_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_project_attempts_user_id_project_id_index ON public.user_project_attempts USING btree (user_id, project_id);


--
-- Name: user_project_attempts_user_id_task_id_created_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_project_attempts_user_id_task_id_created_at_index ON public.user_project_attempts USING btree (user_id, task_id, created_at);


--
-- Name: user_project_attempts_user_id_task_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_project_attempts_user_id_task_id_index ON public.user_project_attempts USING btree (user_id, task_id);


--
-- Name: user_task_progress_status_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_task_progress_status_index ON public.user_task_progress USING btree (status);


--
-- Name: user_task_progress_task_id_status_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_task_progress_task_id_status_index ON public.user_task_progress USING btree (task_id, status);


--
-- Name: user_task_progress_user_id_attempt_group_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_task_progress_user_id_attempt_group_id_index ON public.user_task_progress USING btree (user_id, attempt_group_id);


--
-- Name: user_unlocked_hints_unlocked_at_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_unlocked_hints_unlocked_at_index ON public.user_unlocked_hints USING btree (unlocked_at);


--
-- Name: user_unlocked_hints_user_id_task_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX user_unlocked_hints_user_id_task_id_index ON public.user_unlocked_hints USING btree (user_id, task_id);


--
-- Name: users_newsletter_enabled_newsletter_language_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX users_newsletter_enabled_newsletter_language_index ON public.users USING btree (newsletter_enabled, newsletter_language);


--
-- Name: users_stripe_id_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX users_stripe_id_index ON public.users USING btree (stripe_id);


--
-- Name: votes_subject_key_index; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX votes_subject_key_index ON public.votes USING btree (subject_key);


--
-- Name: coupons coupons_created_by_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.coupons
    ADD CONSTRAINT coupons_created_by_foreign FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: course_translations course_translations_course_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.course_translations
    ADD CONSTRAINT course_translations_course_id_foreign FOREIGN KEY (course_id) REFERENCES public.courses(id) ON DELETE CASCADE;


--
-- Name: gifts gifts_coupon_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gifts
    ADD CONSTRAINT gifts_coupon_id_foreign FOREIGN KEY (coupon_id) REFERENCES public.coupons(id) ON DELETE CASCADE;


--
-- Name: gifts gifts_gifter_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gifts
    ADD CONSTRAINT gifts_gifter_user_id_foreign FOREIGN KEY (gifter_user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: gifts gifts_recipient_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.gifts
    ADD CONSTRAINT gifts_recipient_user_id_foreign FOREIGN KEY (recipient_user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: lab_submissions lab_submissions_lab_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lab_submissions
    ADD CONSTRAINT lab_submissions_lab_id_foreign FOREIGN KEY (lab_id) REFERENCES public.labs(id) ON DELETE CASCADE;


--
-- Name: lab_submissions lab_submissions_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lab_submissions
    ADD CONSTRAINT lab_submissions_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: lesson_bookmarks lesson_bookmarks_lesson_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_bookmarks
    ADD CONSTRAINT lesson_bookmarks_lesson_id_foreign FOREIGN KEY (lesson_id) REFERENCES public.lessons(id) ON DELETE CASCADE;


--
-- Name: lesson_bookmarks lesson_bookmarks_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_bookmarks
    ADD CONSTRAINT lesson_bookmarks_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: lesson_translations lesson_translations_lesson_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lesson_translations
    ADD CONSTRAINT lesson_translations_lesson_id_foreign FOREIGN KEY (lesson_id) REFERENCES public.lessons(id) ON DELETE CASCADE;


--
-- Name: lessons lessons_course_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.lessons
    ADD CONSTRAINT lessons_course_id_foreign FOREIGN KEY (course_id) REFERENCES public.courses(id);


--
-- Name: newsletter_opens newsletter_opens_newsletter_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.newsletter_opens
    ADD CONSTRAINT newsletter_opens_newsletter_id_foreign FOREIGN KEY (newsletter_id) REFERENCES public.newsletters(id) ON DELETE CASCADE;


--
-- Name: newsletter_opens newsletter_opens_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.newsletter_opens
    ADD CONSTRAINT newsletter_opens_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: notes notes_lesson_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notes
    ADD CONSTRAINT notes_lesson_id_foreign FOREIGN KEY (lesson_id) REFERENCES public.lessons(id) ON DELETE CASCADE;


--
-- Name: notes notes_parent_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notes
    ADD CONSTRAINT notes_parent_id_foreign FOREIGN KEY (parent_id) REFERENCES public.notes(id) ON DELETE CASCADE;


--
-- Name: notes notes_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.notes
    ADD CONSTRAINT notes_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: payment_requests payment_requests_reviewed_by_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_requests
    ADD CONSTRAINT payment_requests_reviewed_by_foreign FOREIGN KEY (reviewed_by) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: payment_requests payment_requests_subscription_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_requests
    ADD CONSTRAINT payment_requests_subscription_id_foreign FOREIGN KEY (subscription_id) REFERENCES public.subscriptions(id) ON DELETE SET NULL;


--
-- Name: payment_requests payment_requests_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.payment_requests
    ADD CONSTRAINT payment_requests_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: project_attempt_groups project_attempt_groups_project_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_attempt_groups
    ADD CONSTRAINT project_attempt_groups_project_id_foreign FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;


--
-- Name: project_attempt_groups project_attempt_groups_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_attempt_groups
    ADD CONSTRAINT project_attempt_groups_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: project_translations project_translations_project_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project_translations
    ADD CONSTRAINT project_translations_project_id_foreign FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;


--
-- Name: projects projects_course_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_course_id_foreign FOREIGN KEY (course_id) REFERENCES public.courses(id) ON DELETE SET NULL;


--
-- Name: subscription_items subscription_items_subscription_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscription_items
    ADD CONSTRAINT subscription_items_subscription_id_foreign FOREIGN KEY (subscription_id) REFERENCES public.subscriptions(id) ON DELETE CASCADE;


--
-- Name: subscriptions subscriptions_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.subscriptions
    ADD CONSTRAINT subscriptions_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: task_hints task_hints_task_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_hints
    ADD CONSTRAINT task_hints_task_id_foreign FOREIGN KEY (task_id) REFERENCES public.tasks(id) ON DELETE CASCADE;


--
-- Name: task_translations task_translations_task_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_translations
    ADD CONSTRAINT task_translations_task_id_foreign FOREIGN KEY (task_id) REFERENCES public.tasks(id) ON DELETE CASCADE;


--
-- Name: task_validators task_validators_task_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.task_validators
    ADD CONSTRAINT task_validators_task_id_foreign FOREIGN KEY (task_id) REFERENCES public.tasks(id) ON DELETE CASCADE;


--
-- Name: tasks tasks_project_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tasks
    ADD CONSTRAINT tasks_project_id_foreign FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;


--
-- Name: user_lab_ssh_keys user_lab_ssh_keys_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_lab_ssh_keys
    ADD CONSTRAINT user_lab_ssh_keys_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_project_attempts user_project_attempts_attempt_group_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_project_attempts
    ADD CONSTRAINT user_project_attempts_attempt_group_id_foreign FOREIGN KEY (attempt_group_id) REFERENCES public.project_attempt_groups(id) ON DELETE CASCADE;


--
-- Name: user_project_attempts user_project_attempts_project_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_project_attempts
    ADD CONSTRAINT user_project_attempts_project_id_foreign FOREIGN KEY (project_id) REFERENCES public.projects(id) ON DELETE CASCADE;


--
-- Name: user_project_attempts user_project_attempts_task_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_project_attempts
    ADD CONSTRAINT user_project_attempts_task_id_foreign FOREIGN KEY (task_id) REFERENCES public.tasks(id) ON DELETE CASCADE;


--
-- Name: user_project_attempts user_project_attempts_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_project_attempts
    ADD CONSTRAINT user_project_attempts_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_task_progress user_task_progress_attempt_group_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_task_progress
    ADD CONSTRAINT user_task_progress_attempt_group_id_foreign FOREIGN KEY (attempt_group_id) REFERENCES public.project_attempt_groups(id) ON DELETE CASCADE;


--
-- Name: user_task_progress user_task_progress_task_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_task_progress
    ADD CONSTRAINT user_task_progress_task_id_foreign FOREIGN KEY (task_id) REFERENCES public.tasks(id) ON DELETE CASCADE;


--
-- Name: user_task_progress user_task_progress_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_task_progress
    ADD CONSTRAINT user_task_progress_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_unlocked_hints user_unlocked_hints_task_hint_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_unlocked_hints
    ADD CONSTRAINT user_unlocked_hints_task_hint_id_foreign FOREIGN KEY (task_hint_id) REFERENCES public.task_hints(id) ON DELETE CASCADE;


--
-- Name: user_unlocked_hints user_unlocked_hints_task_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_unlocked_hints
    ADD CONSTRAINT user_unlocked_hints_task_id_foreign FOREIGN KEY (task_id) REFERENCES public.tasks(id) ON DELETE CASCADE;


--
-- Name: user_unlocked_hints user_unlocked_hints_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.user_unlocked_hints
    ADD CONSTRAINT user_unlocked_hints_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: votes votes_user_id_foreign; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.votes
    ADD CONSTRAINT votes_user_id_foreign FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--


