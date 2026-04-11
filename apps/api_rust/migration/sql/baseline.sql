--
-- PostgreSQL database dump
--

-- Dumped from database version 15.7
-- Dumped by pg_dump version 15.7

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

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: accounts; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.accounts (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    provider_account_id character varying(255) NOT NULL,
    provider character varying NOT NULL,
    access_token text NOT NULL,
    access_token_expired_at timestamp with time zone,
    refresh_token text,
    refresh_token_expired_at timestamp with time zone,
    last_connected_at timestamp with time zone NOT NULL,
    metadata jsonb NOT NULL,
    user_id uuid NOT NULL,
    id_token text NOT NULL
);


ALTER TABLE public.accounts OWNER TO plane;

--
-- Name: analytic_views; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.analytic_views (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    query jsonb NOT NULL,
    query_dict jsonb NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.analytic_views OWNER TO plane;

--
-- Name: api_activity_logs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.api_activity_logs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    token_identifier character varying(255) NOT NULL,
    path character varying(255) NOT NULL,
    method character varying(10) NOT NULL,
    query_params text,
    headers text,
    body text,
    response_code integer NOT NULL,
    response_body text,
    ip_address character varying,
    user_agent character varying(512),
    created_by_id uuid,
    updated_by_id uuid,
    deleted_at timestamp with time zone
);


ALTER TABLE public.api_activity_logs OWNER TO plane;

--
-- Name: api_tokens; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.api_tokens (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    token character varying(255) NOT NULL,
    label character varying(255) NOT NULL,
    user_type smallint NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    workspace_id uuid,
    description text NOT NULL,
    expired_at timestamp with time zone,
    is_active boolean NOT NULL,
    last_used timestamp with time zone,
    is_service boolean NOT NULL,
    deleted_at timestamp with time zone,
    allowed_rate_limit character varying(255) NOT NULL
);


ALTER TABLE public.api_tokens OWNER TO plane;

--
-- Name: auth_group; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.auth_group (
    id integer NOT NULL,
    name character varying(150) NOT NULL
);


ALTER TABLE public.auth_group OWNER TO plane;

--
-- Name: auth_group_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.auth_group_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.auth_group_id_seq OWNER TO plane;

--
-- Name: auth_group_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.auth_group_id_seq OWNED BY public.auth_group.id;


--
-- Name: auth_group_permissions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.auth_group_permissions (
    id bigint NOT NULL,
    group_id integer NOT NULL,
    permission_id integer NOT NULL
);


ALTER TABLE public.auth_group_permissions OWNER TO plane;

--
-- Name: auth_group_permissions_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.auth_group_permissions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.auth_group_permissions_id_seq OWNER TO plane;

--
-- Name: auth_group_permissions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.auth_group_permissions_id_seq OWNED BY public.auth_group_permissions.id;


--
-- Name: auth_permission; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.auth_permission (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    content_type_id integer NOT NULL,
    codename character varying(100) NOT NULL
);


ALTER TABLE public.auth_permission OWNER TO plane;

--
-- Name: auth_permission_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.auth_permission_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.auth_permission_id_seq OWNER TO plane;

--
-- Name: auth_permission_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.auth_permission_id_seq OWNED BY public.auth_permission.id;


--
-- Name: changelogs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.changelogs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    title character varying(255) NOT NULL,
    description text NOT NULL,
    version character varying(255) NOT NULL,
    tags jsonb NOT NULL,
    release_date timestamp with time zone,
    is_release_candidate boolean NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    deleted_at timestamp with time zone
);


ALTER TABLE public.changelogs OWNER TO plane;

--
-- Name: comment_reactions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.comment_reactions (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    reaction text NOT NULL,
    actor_id uuid NOT NULL,
    comment_id uuid NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.comment_reactions OWNER TO plane;

--
-- Name: cycle_issues; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.cycle_issues (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    created_by_id uuid,
    cycle_id uuid NOT NULL,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.cycle_issues OWNER TO plane;

--
-- Name: cycle_user_properties; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.cycle_user_properties (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    filters jsonb NOT NULL,
    display_filters jsonb NOT NULL,
    display_properties jsonb NOT NULL,
    created_by_id uuid,
    cycle_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone,
    rich_filters jsonb NOT NULL
);


ALTER TABLE public.cycle_user_properties OWNER TO plane;

--
-- Name: cycles; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.cycles (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    start_date timestamp with time zone,
    end_date timestamp with time zone,
    created_by_id uuid,
    owned_by_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    view_props jsonb NOT NULL,
    sort_order double precision NOT NULL,
    external_id character varying(255),
    external_source character varying(255),
    progress_snapshot jsonb NOT NULL,
    archived_at timestamp with time zone,
    logo_props jsonb NOT NULL,
    deleted_at timestamp with time zone,
    timezone character varying(255) NOT NULL,
    version integer NOT NULL
);


ALTER TABLE public.cycles OWNER TO plane;

--
-- Name: db_githubprstatemapping; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.db_githubprstatemapping (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    github_pr_state character varying(20) NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    workspace_integration_id uuid NOT NULL,
    project_id uuid NOT NULL,
    state_id uuid NOT NULL,
    prevent_regression boolean NOT NULL
);


ALTER TABLE public.db_githubprstatemapping OWNER TO plane;

--
-- Name: deploy_boards; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.deploy_boards (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    entity_identifier uuid,
    entity_name character varying(30),
    anchor character varying(255) NOT NULL,
    is_comments_enabled boolean NOT NULL,
    is_reactions_enabled boolean NOT NULL,
    is_votes_enabled boolean NOT NULL,
    view_props jsonb NOT NULL,
    created_by_id uuid,
    intake_id uuid,
    project_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone,
    is_activity_enabled boolean NOT NULL,
    is_disabled boolean NOT NULL
);


ALTER TABLE public.deploy_boards OWNER TO plane;

--
-- Name: description_versions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.description_versions (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    description_json jsonb NOT NULL,
    description_html text NOT NULL,
    description_binary bytea,
    description_stripped text,
    created_by_id uuid,
    description_id uuid NOT NULL,
    project_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.description_versions OWNER TO plane;

--
-- Name: descriptions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.descriptions (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    description_json jsonb NOT NULL,
    description_html text NOT NULL,
    description_binary bytea,
    description_stripped text,
    created_by_id uuid,
    project_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.descriptions OWNER TO plane;

--
-- Name: device_sessions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.device_sessions (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    is_active boolean NOT NULL,
    user_agent character varying(255),
    ip_address character varying,
    start_time timestamp with time zone NOT NULL,
    end_time timestamp with time zone,
    created_by_id uuid,
    device_id uuid NOT NULL,
    session_id character varying(128) NOT NULL,
    updated_by_id uuid
);


ALTER TABLE public.device_sessions OWNER TO plane;

--
-- Name: devices; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.devices (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    device_id character varying(255),
    device_type character varying(255) NOT NULL,
    push_token character varying(255),
    is_active boolean NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    user_id uuid NOT NULL
);


ALTER TABLE public.devices OWNER TO plane;

--
-- Name: django_celery_beat_clockedschedule; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.django_celery_beat_clockedschedule (
    id integer NOT NULL,
    clocked_time timestamp with time zone NOT NULL
);


ALTER TABLE public.django_celery_beat_clockedschedule OWNER TO plane;

--
-- Name: django_celery_beat_clockedschedule_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.django_celery_beat_clockedschedule_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.django_celery_beat_clockedschedule_id_seq OWNER TO plane;

--
-- Name: django_celery_beat_clockedschedule_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.django_celery_beat_clockedschedule_id_seq OWNED BY public.django_celery_beat_clockedschedule.id;


--
-- Name: django_celery_beat_crontabschedule; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.django_celery_beat_crontabschedule (
    id integer NOT NULL,
    minute character varying(240) NOT NULL,
    hour character varying(96) NOT NULL,
    day_of_week character varying(64) NOT NULL,
    day_of_month character varying(124) NOT NULL,
    month_of_year character varying(64) NOT NULL,
    timezone character varying(63) NOT NULL
);


ALTER TABLE public.django_celery_beat_crontabschedule OWNER TO plane;

--
-- Name: django_celery_beat_crontabschedule_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.django_celery_beat_crontabschedule_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.django_celery_beat_crontabschedule_id_seq OWNER TO plane;

--
-- Name: django_celery_beat_crontabschedule_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.django_celery_beat_crontabschedule_id_seq OWNED BY public.django_celery_beat_crontabschedule.id;


--
-- Name: django_celery_beat_intervalschedule; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.django_celery_beat_intervalschedule (
    id integer NOT NULL,
    every integer NOT NULL,
    period character varying(24) NOT NULL
);


ALTER TABLE public.django_celery_beat_intervalschedule OWNER TO plane;

--
-- Name: django_celery_beat_intervalschedule_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.django_celery_beat_intervalschedule_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.django_celery_beat_intervalschedule_id_seq OWNER TO plane;

--
-- Name: django_celery_beat_intervalschedule_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.django_celery_beat_intervalschedule_id_seq OWNED BY public.django_celery_beat_intervalschedule.id;


--
-- Name: django_celery_beat_periodictask; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.django_celery_beat_periodictask (
    id integer NOT NULL,
    name character varying(200) NOT NULL,
    task character varying(200) NOT NULL,
    args text NOT NULL,
    kwargs text NOT NULL,
    queue character varying(200),
    exchange character varying(200),
    routing_key character varying(200),
    expires timestamp with time zone,
    enabled boolean NOT NULL,
    last_run_at timestamp with time zone,
    total_run_count integer NOT NULL,
    date_changed timestamp with time zone NOT NULL,
    description text NOT NULL,
    crontab_id integer,
    interval_id integer,
    solar_id integer,
    one_off boolean NOT NULL,
    start_time timestamp with time zone,
    priority integer,
    headers text NOT NULL,
    clocked_id integer,
    expire_seconds integer
);


ALTER TABLE public.django_celery_beat_periodictask OWNER TO plane;

--
-- Name: django_celery_beat_periodictask_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.django_celery_beat_periodictask_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.django_celery_beat_periodictask_id_seq OWNER TO plane;

--
-- Name: django_celery_beat_periodictask_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.django_celery_beat_periodictask_id_seq OWNED BY public.django_celery_beat_periodictask.id;


--
-- Name: django_celery_beat_periodictasks; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.django_celery_beat_periodictasks (
    ident smallint NOT NULL,
    last_update timestamp with time zone NOT NULL
);


ALTER TABLE public.django_celery_beat_periodictasks OWNER TO plane;

--
-- Name: django_celery_beat_solarschedule; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.django_celery_beat_solarschedule (
    id integer NOT NULL,
    event character varying(24) NOT NULL,
    latitude numeric(9,6) NOT NULL,
    longitude numeric(9,6) NOT NULL
);


ALTER TABLE public.django_celery_beat_solarschedule OWNER TO plane;

--
-- Name: django_celery_beat_solarschedule_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.django_celery_beat_solarschedule_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.django_celery_beat_solarschedule_id_seq OWNER TO plane;

--
-- Name: django_celery_beat_solarschedule_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.django_celery_beat_solarschedule_id_seq OWNED BY public.django_celery_beat_solarschedule.id;


--
-- Name: django_content_type; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.django_content_type (
    id integer NOT NULL,
    app_label character varying(100) NOT NULL,
    model character varying(100) NOT NULL
);


ALTER TABLE public.django_content_type OWNER TO plane;

--
-- Name: django_content_type_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.django_content_type_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.django_content_type_id_seq OWNER TO plane;

--
-- Name: django_content_type_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.django_content_type_id_seq OWNED BY public.django_content_type.id;


--
-- Name: django_migrations; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.django_migrations (
    id bigint NOT NULL,
    app character varying(255) NOT NULL,
    name character varying(255) NOT NULL,
    applied timestamp with time zone NOT NULL
);


ALTER TABLE public.django_migrations OWNER TO plane;

--
-- Name: django_migrations_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.django_migrations_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.django_migrations_id_seq OWNER TO plane;

--
-- Name: django_migrations_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.django_migrations_id_seq OWNED BY public.django_migrations.id;


--
-- Name: django_session; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.django_session (
    session_key character varying(40) NOT NULL,
    session_data text NOT NULL,
    expire_date timestamp with time zone NOT NULL
);


ALTER TABLE public.django_session OWNER TO plane;

--
-- Name: draft_issue_assignees; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.draft_issue_assignees (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    assignee_id uuid NOT NULL,
    created_by_id uuid,
    draft_issue_id uuid NOT NULL,
    project_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.draft_issue_assignees OWNER TO plane;

--
-- Name: draft_issue_cycles; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.draft_issue_cycles (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    cycle_id uuid NOT NULL,
    created_by_id uuid,
    draft_issue_id uuid NOT NULL,
    project_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.draft_issue_cycles OWNER TO plane;

--
-- Name: draft_issue_labels; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.draft_issue_labels (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    label_id uuid NOT NULL,
    created_by_id uuid,
    draft_issue_id uuid NOT NULL,
    project_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.draft_issue_labels OWNER TO plane;

--
-- Name: draft_issue_modules; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.draft_issue_modules (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    module_id uuid NOT NULL,
    created_by_id uuid,
    draft_issue_id uuid NOT NULL,
    project_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.draft_issue_modules OWNER TO plane;

--
-- Name: draft_issues; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.draft_issues (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    name character varying(255),
    description_json jsonb NOT NULL,
    description_html text NOT NULL,
    description_stripped text,
    description_binary bytea,
    priority character varying(30) NOT NULL,
    start_date date,
    target_date date,
    sort_order double precision NOT NULL,
    completed_at timestamp with time zone,
    external_source character varying(255),
    external_id character varying(255),
    created_by_id uuid,
    estimate_point_id uuid,
    parent_id uuid,
    project_id uuid,
    state_id uuid,
    type_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.draft_issues OWNER TO plane;

--
-- Name: email_notification_logs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.email_notification_logs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    entity_identifier uuid,
    entity_name character varying(255) NOT NULL,
    data jsonb,
    processed_at timestamp with time zone,
    sent_at timestamp with time zone,
    entity character varying(200) NOT NULL,
    old_value character varying(300),
    new_value character varying(300),
    created_by_id uuid,
    receiver_id uuid NOT NULL,
    triggered_by_id uuid NOT NULL,
    updated_by_id uuid,
    deleted_at timestamp with time zone
);


ALTER TABLE public.email_notification_logs OWNER TO plane;

--
-- Name: estimate_points; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.estimate_points (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    key integer NOT NULL,
    description text NOT NULL,
    value character varying(255) NOT NULL,
    created_by_id uuid,
    estimate_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.estimate_points OWNER TO plane;

--
-- Name: estimates; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.estimates (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    type character varying(255) NOT NULL,
    last_used boolean NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.estimates OWNER TO plane;

--
-- Name: exporters; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.exporters (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    provider character varying(50) NOT NULL,
    status character varying(50) NOT NULL,
    reason text NOT NULL,
    key text NOT NULL,
    url character varying(800),
    token character varying(255) NOT NULL,
    created_by_id uuid,
    initiated_by_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    filters jsonb,
    name character varying(255),
    type character varying(50) NOT NULL,
    deleted_at timestamp with time zone,
    rich_filters jsonb
);


ALTER TABLE public.exporters OWNER TO plane;

--
-- Name: file_assets; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.file_assets (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    attributes jsonb NOT NULL,
    asset character varying(800) NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    workspace_id uuid,
    is_deleted boolean NOT NULL,
    deleted_at timestamp with time zone,
    is_archived boolean NOT NULL,
    comment_id uuid,
    entity_type character varying(255),
    external_id character varying(255),
    external_source character varying(255),
    is_uploaded boolean NOT NULL,
    issue_id uuid,
    page_id uuid,
    project_id uuid,
    size double precision NOT NULL,
    storage_metadata jsonb,
    user_id uuid,
    draft_issue_id uuid,
    entity_identifier character varying(255)
);


ALTER TABLE public.file_assets OWNER TO plane;

--
-- Name: github_comment_syncs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.github_comment_syncs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    repo_comment_id bigint NOT NULL,
    comment_id uuid NOT NULL,
    created_by_id uuid,
    issue_sync_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.github_comment_syncs OWNER TO plane;

--
-- Name: github_issue_syncs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.github_issue_syncs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    repo_issue_id bigint NOT NULL,
    github_issue_id bigint NOT NULL,
    issue_url character varying(200) NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    repository_sync_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.github_issue_syncs OWNER TO plane;

--
-- Name: github_repositories; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.github_repositories (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(500) NOT NULL,
    url character varying(200),
    config jsonb NOT NULL,
    repository_id bigint NOT NULL,
    owner character varying(500) NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.github_repositories OWNER TO plane;

--
-- Name: github_repository_syncs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.github_repository_syncs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    credentials jsonb NOT NULL,
    actor_id uuid NOT NULL,
    created_by_id uuid,
    label_id uuid,
    project_id uuid NOT NULL,
    repository_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    workspace_integration_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.github_repository_syncs OWNER TO plane;

--
-- Name: gitlab_comment_syncs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.gitlab_comment_syncs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    repo_comment_id bigint NOT NULL,
    comment_id uuid NOT NULL,
    created_by_id uuid,
    issue_sync_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.gitlab_comment_syncs OWNER TO plane;

--
-- Name: gitlab_issue_syncs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.gitlab_issue_syncs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    repo_issue_id bigint NOT NULL,
    gitlab_issue_id bigint NOT NULL,
    issue_url character varying(200) NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    repository_sync_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.gitlab_issue_syncs OWNER TO plane;

--
-- Name: gitlab_repositories; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.gitlab_repositories (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(500) NOT NULL,
    url character varying(200),
    config jsonb NOT NULL,
    repository_id bigint NOT NULL,
    owner character varying(500) NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.gitlab_repositories OWNER TO plane;

--
-- Name: gitlab_repository_syncs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.gitlab_repository_syncs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    credentials jsonb NOT NULL,
    actor_id uuid NOT NULL,
    created_by_id uuid,
    label_id uuid,
    project_id uuid NOT NULL,
    repository_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    workspace_integration_id uuid NOT NULL
);


ALTER TABLE public.gitlab_repository_syncs OWNER TO plane;

--
-- Name: importers; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.importers (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    service character varying(50) NOT NULL,
    status character varying(50) NOT NULL,
    metadata jsonb NOT NULL,
    config jsonb NOT NULL,
    data jsonb NOT NULL,
    created_by_id uuid,
    initiated_by_id uuid NOT NULL,
    project_id uuid NOT NULL,
    token_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    imported_data jsonb,
    deleted_at timestamp with time zone
);


ALTER TABLE public.importers OWNER TO plane;

--
-- Name: instance_admins; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.instance_admins (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    role integer NOT NULL,
    is_verified boolean NOT NULL,
    created_by_id uuid,
    instance_id uuid NOT NULL,
    updated_by_id uuid,
    user_id uuid,
    deleted_at timestamp with time zone
);


ALTER TABLE public.instance_admins OWNER TO plane;

--
-- Name: instance_configurations; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.instance_configurations (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    key character varying(100) NOT NULL,
    value text,
    category text NOT NULL,
    is_encrypted boolean NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    deleted_at timestamp with time zone
);


ALTER TABLE public.instance_configurations OWNER TO plane;

--
-- Name: instances; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.instances (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    instance_name character varying(255) NOT NULL,
    whitelist_emails text,
    instance_id character varying(255) NOT NULL,
    current_version character varying(255) NOT NULL,
    last_checked_at timestamp with time zone NOT NULL,
    namespace character varying(255),
    is_telemetry_enabled boolean NOT NULL,
    is_support_required boolean NOT NULL,
    is_setup_done boolean NOT NULL,
    is_signup_screen_visited boolean NOT NULL,
    is_verified boolean NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    domain text NOT NULL,
    latest_version character varying(255),
    edition character varying(255) NOT NULL,
    deleted_at timestamp with time zone,
    is_test boolean NOT NULL,
    is_current_version_deprecated boolean NOT NULL
);


ALTER TABLE public.instances OWNER TO plane;

--
-- Name: intake_issues; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.intake_issues (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    status integer NOT NULL,
    snoozed_till timestamp with time zone,
    source character varying(255),
    created_by_id uuid,
    duplicate_to_id uuid,
    intake_id uuid NOT NULL,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    external_id character varying(255),
    external_source character varying(255),
    deleted_at timestamp with time zone,
    extra jsonb NOT NULL,
    source_email text
);


ALTER TABLE public.intake_issues OWNER TO plane;

--
-- Name: intakes; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.intakes (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    is_default boolean NOT NULL,
    view_props jsonb NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    logo_props jsonb NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.intakes OWNER TO plane;

--
-- Name: integrations; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.integrations (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    title character varying(400) NOT NULL,
    provider character varying(400) NOT NULL,
    network integer NOT NULL,
    description jsonb NOT NULL,
    author character varying(400) NOT NULL,
    webhook_url text NOT NULL,
    webhook_secret text NOT NULL,
    redirect_url text NOT NULL,
    metadata jsonb NOT NULL,
    verified boolean NOT NULL,
    avatar_url text,
    created_by_id uuid,
    updated_by_id uuid,
    deleted_at timestamp with time zone
);


ALTER TABLE public.integrations OWNER TO plane;

--
-- Name: issue_activities; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_activities (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    verb character varying(255) NOT NULL,
    field character varying(255),
    old_value text,
    new_value text,
    comment text NOT NULL,
    created_by_id uuid,
    issue_id uuid,
    issue_comment_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    actor_id uuid,
    new_identifier uuid,
    old_identifier uuid,
    epoch double precision,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_activities OWNER TO plane;

--
-- Name: issue_assignees; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_assignees (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    assignee_id uuid NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_assignees OWNER TO plane;

--
-- Name: issue_attachments; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_attachments (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    attributes jsonb NOT NULL,
    asset character varying(100) NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    external_id character varying(255),
    external_source character varying(255),
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_attachments OWNER TO plane;

--
-- Name: issue_blockers; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_blockers (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    block_id uuid NOT NULL,
    blocked_by_id uuid NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_blockers OWNER TO plane;

--
-- Name: issue_comments; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_comments (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    comment_stripped text NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    actor_id uuid,
    comment_html text NOT NULL,
    comment_json jsonb NOT NULL,
    access character varying(100) NOT NULL,
    external_id character varying(255),
    external_source character varying(255),
    deleted_at timestamp with time zone,
    edited_at timestamp with time zone,
    description_id uuid,
    parent_id uuid
);


ALTER TABLE public.issue_comments OWNER TO plane;

--
-- Name: issue_description_versions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_description_versions (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    description_binary bytea,
    description_html text NOT NULL,
    description_stripped text,
    description_json jsonb NOT NULL,
    last_saved_at timestamp with time zone NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    owned_by_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.issue_description_versions OWNER TO plane;

--
-- Name: issue_labels; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_labels (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    label_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_labels OWNER TO plane;

--
-- Name: issue_links; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_links (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    title character varying(255),
    url text NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    metadata jsonb NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_links OWNER TO plane;

--
-- Name: issue_mentions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_mentions (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    mention_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_mentions OWNER TO plane;

--
-- Name: issue_reactions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_reactions (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    reaction text NOT NULL,
    actor_id uuid NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_reactions OWNER TO plane;

--
-- Name: issue_relations; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_relations (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    relation_type character varying(20) NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    related_issue_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_relations OWNER TO plane;

--
-- Name: issue_sequences; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_sequences (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    sequence bigint NOT NULL,
    deleted boolean NOT NULL,
    created_by_id uuid,
    issue_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_sequences OWNER TO plane;

--
-- Name: issue_subscribers; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_subscribers (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    subscriber_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_subscribers OWNER TO plane;

--
-- Name: issue_types; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_types (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    logo_props jsonb NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    is_active boolean NOT NULL,
    deleted_at timestamp with time zone,
    is_default boolean NOT NULL,
    level double precision NOT NULL,
    external_id character varying(255),
    external_source character varying(255),
    is_epic boolean NOT NULL
);


ALTER TABLE public.issue_types OWNER TO plane;

--
-- Name: issue_versions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_versions (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    parent uuid,
    state uuid,
    estimate_point uuid,
    name character varying(255) NOT NULL,
    priority character varying(30) NOT NULL,
    start_date date,
    target_date date,
    sequence_id integer NOT NULL,
    sort_order double precision NOT NULL,
    completed_at timestamp with time zone,
    archived_at date,
    is_draft boolean NOT NULL,
    external_source character varying(255),
    external_id character varying(255),
    type uuid,
    last_saved_at timestamp with time zone NOT NULL,
    owned_by_id uuid NOT NULL,
    properties jsonb NOT NULL,
    meta jsonb NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    activity_id uuid
);


ALTER TABLE public.issue_versions OWNER TO plane;

--
-- Name: issue_views; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_views (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    query jsonb NOT NULL,
    access smallint NOT NULL,
    filters jsonb NOT NULL,
    created_by_id uuid,
    project_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    display_filters jsonb NOT NULL,
    display_properties jsonb NOT NULL,
    sort_order double precision NOT NULL,
    logo_props jsonb NOT NULL,
    is_locked boolean NOT NULL,
    owned_by_id uuid NOT NULL,
    deleted_at timestamp with time zone,
    rich_filters jsonb NOT NULL,
    archived_at timestamp with time zone
);


ALTER TABLE public.issue_views OWNER TO plane;

--
-- Name: issue_votes; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issue_votes (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    vote integer NOT NULL,
    actor_id uuid NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issue_votes OWNER TO plane;

--
-- Name: issues; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.issues (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description_json jsonb NOT NULL,
    priority character varying(30) NOT NULL,
    start_date date,
    target_date date,
    sequence_id integer NOT NULL,
    created_by_id uuid,
    parent_id uuid,
    project_id uuid NOT NULL,
    state_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    description_html text NOT NULL,
    description_stripped text,
    completed_at timestamp with time zone,
    sort_order double precision NOT NULL,
    point integer,
    archived_at date,
    is_draft boolean NOT NULL,
    external_id character varying(255),
    external_source character varying(255),
    description_binary bytea,
    estimate_point_id uuid,
    type_id uuid,
    deleted_at timestamp with time zone
);


ALTER TABLE public.issues OWNER TO plane;

--
-- Name: labels; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.labels (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    created_by_id uuid,
    project_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    parent_id uuid,
    color character varying(255) NOT NULL,
    sort_order double precision NOT NULL,
    external_id character varying(255),
    external_source character varying(255),
    deleted_at timestamp with time zone
);


ALTER TABLE public.labels OWNER TO plane;

--
-- Name: module_issues; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.module_issues (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    created_by_id uuid,
    issue_id uuid NOT NULL,
    module_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.module_issues OWNER TO plane;

--
-- Name: module_links; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.module_links (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    title character varying(255),
    url character varying(200) NOT NULL,
    created_by_id uuid,
    module_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    metadata jsonb NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.module_links OWNER TO plane;

--
-- Name: module_members; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.module_members (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    created_by_id uuid,
    member_id uuid NOT NULL,
    module_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.module_members OWNER TO plane;

--
-- Name: module_user_properties; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.module_user_properties (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    filters jsonb NOT NULL,
    display_filters jsonb NOT NULL,
    display_properties jsonb NOT NULL,
    created_by_id uuid,
    module_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone,
    rich_filters jsonb NOT NULL
);


ALTER TABLE public.module_user_properties OWNER TO plane;

--
-- Name: modules; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.modules (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    description_text jsonb,
    description_html jsonb,
    start_date date,
    target_date date,
    status character varying(20) NOT NULL,
    created_by_id uuid,
    lead_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    view_props jsonb NOT NULL,
    sort_order double precision NOT NULL,
    external_id character varying(255),
    external_source character varying(255),
    archived_at timestamp with time zone,
    logo_props jsonb NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.modules OWNER TO plane;

--
-- Name: notifications; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.notifications (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    data jsonb,
    entity_identifier uuid,
    entity_name character varying(255) NOT NULL,
    title text NOT NULL,
    message jsonb,
    message_html text NOT NULL,
    message_stripped text,
    sender character varying(255) NOT NULL,
    read_at timestamp with time zone,
    snoozed_till timestamp with time zone,
    archived_at timestamp with time zone,
    created_by_id uuid,
    project_id uuid,
    receiver_id uuid NOT NULL,
    triggered_by_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.notifications OWNER TO plane;

--
-- Name: page_labels; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.page_labels (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    created_by_id uuid,
    label_id uuid NOT NULL,
    page_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.page_labels OWNER TO plane;

--
-- Name: page_logs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.page_logs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    transaction uuid NOT NULL,
    entity_identifier uuid,
    entity_name character varying(30) NOT NULL,
    created_by_id uuid,
    page_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone,
    entity_type character varying(30)
);


ALTER TABLE public.page_logs OWNER TO plane;

--
-- Name: page_versions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.page_versions (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    last_saved_at timestamp with time zone NOT NULL,
    description_binary bytea,
    description_html text NOT NULL,
    description_stripped text,
    description_json jsonb NOT NULL,
    created_by_id uuid,
    owned_by_id uuid NOT NULL,
    page_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone,
    sub_pages_data jsonb NOT NULL
);


ALTER TABLE public.page_versions OWNER TO plane;

--
-- Name: pages; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.pages (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name text NOT NULL,
    description_json jsonb NOT NULL,
    description_html text NOT NULL,
    description_stripped text,
    access smallint NOT NULL,
    created_by_id uuid,
    owned_by_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    color character varying(255) NOT NULL,
    archived_at date,
    is_locked boolean NOT NULL,
    parent_id uuid,
    view_props jsonb NOT NULL,
    logo_props jsonb NOT NULL,
    description_binary bytea,
    is_global boolean NOT NULL,
    deleted_at timestamp with time zone,
    moved_to_page uuid,
    moved_to_project uuid,
    external_id character varying(255),
    external_source character varying(255),
    sort_order double precision NOT NULL
);


ALTER TABLE public.pages OWNER TO plane;

--
-- Name: profiles; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.profiles (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    theme jsonb NOT NULL,
    is_tour_completed boolean NOT NULL,
    onboarding_step jsonb NOT NULL,
    use_case text,
    role character varying(300),
    is_onboarded boolean NOT NULL,
    last_workspace_id uuid,
    billing_address_country character varying(255) NOT NULL,
    billing_address jsonb,
    has_billing_address boolean NOT NULL,
    company_name character varying(255) NOT NULL,
    user_id uuid NOT NULL,
    is_mobile_onboarded boolean NOT NULL,
    mobile_onboarding_step jsonb NOT NULL,
    mobile_timezone_auto_set boolean NOT NULL,
    language character varying(255) NOT NULL,
    is_smooth_cursor_enabled boolean NOT NULL,
    start_of_the_week smallint NOT NULL,
    is_app_rail_docked boolean NOT NULL,
    background_color character varying(255) NOT NULL,
    goals jsonb NOT NULL,
    has_marketing_email_consent boolean NOT NULL,
    is_navigation_tour_completed boolean NOT NULL,
    is_subscribed_to_changelog boolean NOT NULL,
    notification_view_mode character varying(255) NOT NULL,
    product_tour jsonb NOT NULL
);


ALTER TABLE public.profiles OWNER TO plane;

--
-- Name: project_deploy_boards; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.project_deploy_boards (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    anchor character varying(255) NOT NULL,
    comments boolean NOT NULL,
    reactions boolean NOT NULL,
    votes boolean NOT NULL,
    views jsonb NOT NULL,
    created_by_id uuid,
    intake_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.project_deploy_boards OWNER TO plane;

--
-- Name: project_identifiers; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.project_identifiers (
    id bigint NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    name character varying(12) NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid,
    deleted_at timestamp with time zone
);


ALTER TABLE public.project_identifiers OWNER TO plane;

--
-- Name: project_identifiers_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.project_identifiers_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.project_identifiers_id_seq OWNER TO plane;

--
-- Name: project_identifiers_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.project_identifiers_id_seq OWNED BY public.project_identifiers.id;


--
-- Name: project_issue_types; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.project_issue_types (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    level integer NOT NULL,
    is_default boolean NOT NULL,
    created_by_id uuid,
    issue_type_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.project_issue_types OWNER TO plane;

--
-- Name: project_member_invites; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.project_member_invites (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    email character varying(255) NOT NULL,
    accepted boolean NOT NULL,
    token character varying(255) NOT NULL,
    message text,
    responded_at timestamp with time zone,
    role smallint NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.project_member_invites OWNER TO plane;

--
-- Name: project_members; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.project_members (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    comment text,
    role smallint NOT NULL,
    created_by_id uuid,
    member_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    view_props jsonb NOT NULL,
    default_props jsonb NOT NULL,
    sort_order double precision NOT NULL,
    preferences jsonb NOT NULL,
    is_active boolean NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.project_members OWNER TO plane;

--
-- Name: project_pages; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.project_pages (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    created_by_id uuid,
    page_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.project_pages OWNER TO plane;

--
-- Name: project_public_members; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.project_public_members (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    created_by_id uuid,
    member_id uuid NOT NULL,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.project_public_members OWNER TO plane;

--
-- Name: project_user_properties; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.project_user_properties (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    display_properties jsonb NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    display_filters jsonb NOT NULL,
    filters jsonb NOT NULL,
    deleted_at timestamp with time zone,
    rich_filters jsonb NOT NULL,
    preferences jsonb NOT NULL,
    sort_order double precision NOT NULL
);


ALTER TABLE public.project_user_properties OWNER TO plane;

--
-- Name: project_webhooks; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.project_webhooks (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    webhook_id uuid NOT NULL,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.project_webhooks OWNER TO plane;

--
-- Name: projects; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.projects (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    description_text jsonb,
    description_html jsonb,
    network smallint NOT NULL,
    identifier character varying(12) NOT NULL,
    created_by_id uuid,
    default_assignee_id uuid,
    project_lead_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    emoji character varying(255),
    cycle_view boolean NOT NULL,
    module_view boolean NOT NULL,
    cover_image text,
    issue_views_view boolean NOT NULL,
    page_view boolean NOT NULL,
    estimate_id uuid,
    icon_prop jsonb,
    intake_view boolean NOT NULL,
    archive_in integer NOT NULL,
    close_in integer NOT NULL,
    default_state_id uuid,
    logo_props jsonb NOT NULL,
    archived_at timestamp with time zone,
    is_time_tracking_enabled boolean NOT NULL,
    is_issue_type_enabled boolean NOT NULL,
    deleted_at timestamp with time zone,
    guest_view_all_features boolean NOT NULL,
    timezone character varying(255) NOT NULL,
    cover_image_asset_id uuid,
    external_id character varying(255),
    external_source character varying(255)
);


ALTER TABLE public.projects OWNER TO plane;

--
-- Name: seaql_migrations; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.seaql_migrations (
    version character varying NOT NULL,
    applied_at bigint NOT NULL
);


ALTER TABLE public.seaql_migrations OWNER TO plane;

--
-- Name: sessions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.sessions (
    session_key character varying(128) NOT NULL,
    session_data text NOT NULL,
    expire_date timestamp with time zone NOT NULL,
    device_info jsonb,
    user_id character varying(50)
);


ALTER TABLE public.sessions OWNER TO plane;

--
-- Name: slack_project_syncs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.slack_project_syncs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    access_token character varying(300) NOT NULL,
    scopes text NOT NULL,
    bot_user_id character varying(50) NOT NULL,
    webhook_url character varying(1000) NOT NULL,
    data jsonb NOT NULL,
    team_id character varying(30) NOT NULL,
    team_name character varying(300) NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    workspace_integration_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.slack_project_syncs OWNER TO plane;

--
-- Name: social_login_connections; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.social_login_connections (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    medium character varying(20) NOT NULL,
    last_login_at timestamp with time zone,
    last_received_at timestamp with time zone,
    token_data jsonb,
    extra_data jsonb,
    created_by_id uuid,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.social_login_connections OWNER TO plane;

--
-- Name: states; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.states (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    color character varying(255) NOT NULL,
    slug character varying(100) NOT NULL,
    created_by_id uuid,
    project_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    sequence double precision NOT NULL,
    "group" character varying(20) NOT NULL,
    "default" boolean NOT NULL,
    external_id character varying(255),
    external_source character varying(255),
    is_triage boolean NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.states OWNER TO plane;

--
-- Name: stickies; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.stickies (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    name text,
    description jsonb NOT NULL,
    description_html text NOT NULL,
    description_stripped text,
    description_binary bytea,
    logo_props jsonb NOT NULL,
    color character varying(255),
    background_color character varying(255),
    created_by_id uuid,
    owner_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    sort_order double precision NOT NULL
);


ALTER TABLE public.stickies OWNER TO plane;

--
-- Name: teams; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.teams (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    logo_props jsonb NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.teams OWNER TO plane;

--
-- Name: user_favorites; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.user_favorites (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    entity_type character varying(100) NOT NULL,
    entity_identifier uuid,
    name character varying(255),
    is_folder boolean NOT NULL,
    sequence double precision NOT NULL,
    created_by_id uuid,
    parent_id uuid,
    project_id uuid,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.user_favorites OWNER TO plane;

--
-- Name: user_github_connections; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.user_github_connections (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    github_user_id character varying(100) NOT NULL,
    github_username character varying(150) NOT NULL,
    github_avatar_url character varying(200) NOT NULL,
    access_token text NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    user_id uuid NOT NULL
);


ALTER TABLE public.user_github_connections OWNER TO plane;

--
-- Name: user_notification_preferences; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.user_notification_preferences (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    property_change boolean NOT NULL,
    state_change boolean NOT NULL,
    comment boolean NOT NULL,
    mention boolean NOT NULL,
    issue_completed boolean NOT NULL,
    created_by_id uuid,
    project_id uuid,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    workspace_id uuid,
    deleted_at timestamp with time zone
);


ALTER TABLE public.user_notification_preferences OWNER TO plane;

--
-- Name: user_recent_visits; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.user_recent_visits (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    entity_identifier uuid,
    entity_name character varying(30) NOT NULL,
    visited_at timestamp with time zone NOT NULL,
    created_by_id uuid,
    project_id uuid,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.user_recent_visits OWNER TO plane;

--
-- Name: users; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.users (
    id uuid NOT NULL,
    password character varying(128) NOT NULL,
    last_login timestamp with time zone,
    username character varying(128) NOT NULL,
    mobile_number character varying(255),
    email character varying(255),
    first_name character varying(255) NOT NULL,
    last_name character varying(255) NOT NULL,
    avatar text NOT NULL,
    date_joined timestamp with time zone NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    last_location character varying(255) NOT NULL,
    created_location character varying(255) NOT NULL,
    is_superuser boolean NOT NULL,
    is_managed boolean NOT NULL,
    is_password_expired boolean NOT NULL,
    is_active boolean NOT NULL,
    is_staff boolean NOT NULL,
    is_email_verified boolean NOT NULL,
    is_password_autoset boolean NOT NULL,
    token character varying(64) NOT NULL,
    user_timezone character varying(255) NOT NULL,
    last_active timestamp with time zone,
    last_login_time timestamp with time zone,
    last_logout_time timestamp with time zone,
    last_login_ip character varying(255) NOT NULL,
    last_logout_ip character varying(255) NOT NULL,
    last_login_medium character varying(20) NOT NULL,
    last_login_uagent text NOT NULL,
    token_updated_at timestamp with time zone,
    is_bot boolean NOT NULL,
    cover_image character varying(800),
    display_name character varying(255) NOT NULL,
    avatar_asset_id uuid,
    cover_image_asset_id uuid,
    bot_type character varying(30),
    is_email_valid boolean NOT NULL,
    masked_at timestamp with time zone,
    is_password_reset_required boolean NOT NULL
);


ALTER TABLE public.users OWNER TO plane;

--
-- Name: users_groups; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.users_groups (
    id bigint NOT NULL,
    user_id uuid NOT NULL,
    group_id integer NOT NULL
);


ALTER TABLE public.users_groups OWNER TO plane;

--
-- Name: users_groups_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.users_groups_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.users_groups_id_seq OWNER TO plane;

--
-- Name: users_groups_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.users_groups_id_seq OWNED BY public.users_groups.id;


--
-- Name: users_user_permissions; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.users_user_permissions (
    id bigint NOT NULL,
    user_id uuid NOT NULL,
    permission_id integer NOT NULL
);


ALTER TABLE public.users_user_permissions OWNER TO plane;

--
-- Name: users_user_permissions_id_seq; Type: SEQUENCE; Schema: public; Owner: plane
--

CREATE SEQUENCE public.users_user_permissions_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.users_user_permissions_id_seq OWNER TO plane;

--
-- Name: users_user_permissions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: plane
--

ALTER SEQUENCE public.users_user_permissions_id_seq OWNED BY public.users_user_permissions.id;


--
-- Name: webhook_logs; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.webhook_logs (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    event_type character varying(255),
    request_method character varying(10),
    request_headers text,
    request_body text,
    response_status text,
    response_headers text,
    response_body text,
    retry_count smallint NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    webhook uuid NOT NULL,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.webhook_logs OWNER TO plane;

--
-- Name: webhooks; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.webhooks (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    url character varying(1024) NOT NULL,
    is_active boolean NOT NULL,
    secret_key character varying(255) NOT NULL,
    project boolean NOT NULL,
    issue boolean NOT NULL,
    module boolean NOT NULL,
    cycle boolean NOT NULL,
    issue_comment boolean NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone,
    is_internal boolean NOT NULL,
    version character varying(50) NOT NULL
);


ALTER TABLE public.webhooks OWNER TO plane;

--
-- Name: workspace_home_preferences; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.workspace_home_preferences (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    key character varying(255) NOT NULL,
    is_enabled boolean NOT NULL,
    config jsonb NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    sort_order double precision NOT NULL
);


ALTER TABLE public.workspace_home_preferences OWNER TO plane;

--
-- Name: workspace_integrations; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.workspace_integrations (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    metadata jsonb NOT NULL,
    config jsonb NOT NULL,
    actor_id uuid NOT NULL,
    api_token_id uuid NOT NULL,
    created_by_id uuid,
    integration_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.workspace_integrations OWNER TO plane;

--
-- Name: workspace_member_invites; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.workspace_member_invites (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    email character varying(255) NOT NULL,
    accepted boolean NOT NULL,
    token character varying(255) NOT NULL,
    message text,
    responded_at timestamp with time zone,
    role smallint NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.workspace_member_invites OWNER TO plane;

--
-- Name: workspace_members; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.workspace_members (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    role smallint NOT NULL,
    created_by_id uuid,
    member_id uuid NOT NULL,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    company_role text,
    view_props jsonb NOT NULL,
    default_props jsonb NOT NULL,
    issue_props jsonb NOT NULL,
    is_active boolean NOT NULL,
    deleted_at timestamp with time zone,
    explored_features jsonb NOT NULL,
    getting_started_checklist jsonb NOT NULL,
    tips jsonb NOT NULL
);


ALTER TABLE public.workspace_members OWNER TO plane;

--
-- Name: workspace_themes; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.workspace_themes (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(300) NOT NULL,
    colors jsonb NOT NULL,
    actor_id uuid NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone
);


ALTER TABLE public.workspace_themes OWNER TO plane;

--
-- Name: workspace_user_links; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.workspace_user_links (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    title character varying(255),
    url text NOT NULL,
    metadata jsonb NOT NULL,
    created_by_id uuid,
    owner_id uuid NOT NULL,
    project_id uuid,
    updated_by_id uuid,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.workspace_user_links OWNER TO plane;

--
-- Name: workspace_user_preferences; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.workspace_user_preferences (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone,
    id uuid NOT NULL,
    key character varying(255) NOT NULL,
    is_pinned boolean NOT NULL,
    sort_order double precision NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    workspace_id uuid NOT NULL
);


ALTER TABLE public.workspace_user_preferences OWNER TO plane;

--
-- Name: workspace_user_properties; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.workspace_user_properties (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    filters jsonb NOT NULL,
    display_filters jsonb NOT NULL,
    display_properties jsonb NOT NULL,
    created_by_id uuid,
    updated_by_id uuid,
    user_id uuid NOT NULL,
    workspace_id uuid NOT NULL,
    deleted_at timestamp with time zone,
    rich_filters jsonb NOT NULL,
    navigation_control_preference character varying(25) NOT NULL,
    navigation_project_limit integer NOT NULL
);


ALTER TABLE public.workspace_user_properties OWNER TO plane;

--
-- Name: workspaces; Type: TABLE; Schema: public; Owner: plane
--

CREATE TABLE public.workspaces (
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    id uuid NOT NULL,
    name character varying(80) NOT NULL,
    logo text,
    slug character varying(48) NOT NULL,
    created_by_id uuid,
    owner_id uuid NOT NULL,
    updated_by_id uuid,
    organization_size character varying(20),
    deleted_at timestamp with time zone,
    logo_asset_id uuid,
    timezone character varying(255) NOT NULL,
    background_color character varying(255) NOT NULL
);


ALTER TABLE public.workspaces OWNER TO plane;

--
-- Name: auth_group id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.auth_group ALTER COLUMN id SET DEFAULT nextval('public.auth_group_id_seq'::regclass);


--
-- Name: auth_group_permissions id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.auth_group_permissions ALTER COLUMN id SET DEFAULT nextval('public.auth_group_permissions_id_seq'::regclass);


--
-- Name: auth_permission id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.auth_permission ALTER COLUMN id SET DEFAULT nextval('public.auth_permission_id_seq'::regclass);


--
-- Name: django_celery_beat_clockedschedule id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_clockedschedule ALTER COLUMN id SET DEFAULT nextval('public.django_celery_beat_clockedschedule_id_seq'::regclass);


--
-- Name: django_celery_beat_crontabschedule id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_crontabschedule ALTER COLUMN id SET DEFAULT nextval('public.django_celery_beat_crontabschedule_id_seq'::regclass);


--
-- Name: django_celery_beat_intervalschedule id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_intervalschedule ALTER COLUMN id SET DEFAULT nextval('public.django_celery_beat_intervalschedule_id_seq'::regclass);


--
-- Name: django_celery_beat_periodictask id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_periodictask ALTER COLUMN id SET DEFAULT nextval('public.django_celery_beat_periodictask_id_seq'::regclass);


--
-- Name: django_celery_beat_solarschedule id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_solarschedule ALTER COLUMN id SET DEFAULT nextval('public.django_celery_beat_solarschedule_id_seq'::regclass);


--
-- Name: django_content_type id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_content_type ALTER COLUMN id SET DEFAULT nextval('public.django_content_type_id_seq'::regclass);


--
-- Name: django_migrations id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_migrations ALTER COLUMN id SET DEFAULT nextval('public.django_migrations_id_seq'::regclass);


--
-- Name: project_identifiers id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_identifiers ALTER COLUMN id SET DEFAULT nextval('public.project_identifiers_id_seq'::regclass);


--
-- Name: users_groups id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users_groups ALTER COLUMN id SET DEFAULT nextval('public.users_groups_id_seq'::regclass);


--
-- Name: users_user_permissions id; Type: DEFAULT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users_user_permissions ALTER COLUMN id SET DEFAULT nextval('public.users_user_permissions_id_seq'::regclass);


--
-- Name: accounts accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT accounts_pkey PRIMARY KEY (id);


--
-- Name: analytic_views analytic_views_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.analytic_views
    ADD CONSTRAINT analytic_views_pkey PRIMARY KEY (id);


--
-- Name: api_activity_logs api_activity_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.api_activity_logs
    ADD CONSTRAINT api_activity_logs_pkey PRIMARY KEY (id);


--
-- Name: api_tokens api_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT api_tokens_pkey PRIMARY KEY (id);


--
-- Name: api_tokens api_tokens_token_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT api_tokens_token_key UNIQUE (token);


--
-- Name: auth_group auth_group_name_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.auth_group
    ADD CONSTRAINT auth_group_name_key UNIQUE (name);


--
-- Name: auth_group_permissions auth_group_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.auth_group_permissions
    ADD CONSTRAINT auth_group_permissions_pkey PRIMARY KEY (id);


--
-- Name: auth_group auth_group_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.auth_group
    ADD CONSTRAINT auth_group_pkey PRIMARY KEY (id);


--
-- Name: auth_permission auth_permission_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.auth_permission
    ADD CONSTRAINT auth_permission_pkey PRIMARY KEY (id);


--
-- Name: changelogs changelogs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.changelogs
    ADD CONSTRAINT changelogs_pkey PRIMARY KEY (id);


--
-- Name: comment_reactions comment_reactions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_pkey PRIMARY KEY (id);


--
-- Name: cycle_issues cycle_issues_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_issues
    ADD CONSTRAINT cycle_issues_pkey PRIMARY KEY (id);


--
-- Name: cycle_user_properties cycle_user_properties_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_user_properties
    ADD CONSTRAINT cycle_user_properties_pkey PRIMARY KEY (id);


--
-- Name: cycles cycles_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycles
    ADD CONSTRAINT cycles_pkey PRIMARY KEY (id);


--
-- Name: db_githubprstatemapping db_githubprstatemapping_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.db_githubprstatemapping
    ADD CONSTRAINT db_githubprstatemapping_pkey PRIMARY KEY (id);


--
-- Name: deploy_boards deploy_boards_anchor_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.deploy_boards
    ADD CONSTRAINT deploy_boards_anchor_key UNIQUE (anchor);


--
-- Name: deploy_boards deploy_boards_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.deploy_boards
    ADD CONSTRAINT deploy_boards_pkey PRIMARY KEY (id);


--
-- Name: description_versions description_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.description_versions
    ADD CONSTRAINT description_versions_pkey PRIMARY KEY (id);


--
-- Name: descriptions descriptions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.descriptions
    ADD CONSTRAINT descriptions_pkey PRIMARY KEY (id);


--
-- Name: device_sessions device_sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.device_sessions
    ADD CONSTRAINT device_sessions_pkey PRIMARY KEY (id);


--
-- Name: devices devices_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_pkey PRIMARY KEY (id);


--
-- Name: django_celery_beat_clockedschedule django_celery_beat_clockedschedule_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_clockedschedule
    ADD CONSTRAINT django_celery_beat_clockedschedule_pkey PRIMARY KEY (id);


--
-- Name: django_celery_beat_crontabschedule django_celery_beat_crontabschedule_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_crontabschedule
    ADD CONSTRAINT django_celery_beat_crontabschedule_pkey PRIMARY KEY (id);


--
-- Name: django_celery_beat_intervalschedule django_celery_beat_intervalschedule_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_intervalschedule
    ADD CONSTRAINT django_celery_beat_intervalschedule_pkey PRIMARY KEY (id);


--
-- Name: django_celery_beat_periodictask django_celery_beat_periodictask_name_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_periodictask
    ADD CONSTRAINT django_celery_beat_periodictask_name_key UNIQUE (name);


--
-- Name: django_celery_beat_periodictask django_celery_beat_periodictask_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_periodictask
    ADD CONSTRAINT django_celery_beat_periodictask_pkey PRIMARY KEY (id);


--
-- Name: django_celery_beat_periodictasks django_celery_beat_periodictasks_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_periodictasks
    ADD CONSTRAINT django_celery_beat_periodictasks_pkey PRIMARY KEY (ident);


--
-- Name: django_celery_beat_solarschedule django_celery_beat_solarschedule_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_solarschedule
    ADD CONSTRAINT django_celery_beat_solarschedule_pkey PRIMARY KEY (id);


--
-- Name: django_content_type django_content_type_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_content_type
    ADD CONSTRAINT django_content_type_pkey PRIMARY KEY (id);


--
-- Name: django_migrations django_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_migrations
    ADD CONSTRAINT django_migrations_pkey PRIMARY KEY (id);


--
-- Name: django_session django_session_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_session
    ADD CONSTRAINT django_session_pkey PRIMARY KEY (session_key);


--
-- Name: draft_issue_assignees draft_issue_assignees_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_assignees
    ADD CONSTRAINT draft_issue_assignees_pkey PRIMARY KEY (id);


--
-- Name: draft_issue_cycles draft_issue_cycles_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_cycles
    ADD CONSTRAINT draft_issue_cycles_pkey PRIMARY KEY (id);


--
-- Name: draft_issue_labels draft_issue_labels_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_labels
    ADD CONSTRAINT draft_issue_labels_pkey PRIMARY KEY (id);


--
-- Name: draft_issue_modules draft_issue_modules_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_modules
    ADD CONSTRAINT draft_issue_modules_pkey PRIMARY KEY (id);


--
-- Name: draft_issues draft_issues_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issues
    ADD CONSTRAINT draft_issues_pkey PRIMARY KEY (id);


--
-- Name: email_notification_logs email_notification_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.email_notification_logs
    ADD CONSTRAINT email_notification_logs_pkey PRIMARY KEY (id);


--
-- Name: estimate_points estimate_points_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimate_points
    ADD CONSTRAINT estimate_points_pkey PRIMARY KEY (id);


--
-- Name: estimates estimates_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimates
    ADD CONSTRAINT estimates_pkey PRIMARY KEY (id);


--
-- Name: exporters exporters_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.exporters
    ADD CONSTRAINT exporters_pkey PRIMARY KEY (id);


--
-- Name: exporters exporters_token_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.exporters
    ADD CONSTRAINT exporters_token_key UNIQUE (token);


--
-- Name: file_assets file_assets_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.file_assets
    ADD CONSTRAINT file_assets_pkey PRIMARY KEY (id);


--
-- Name: github_comment_syncs github_comment_syncs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_comment_syncs
    ADD CONSTRAINT github_comment_syncs_pkey PRIMARY KEY (id);


--
-- Name: github_issue_syncs github_issue_syncs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_issue_syncs
    ADD CONSTRAINT github_issue_syncs_pkey PRIMARY KEY (id);


--
-- Name: github_repositories github_repositories_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repositories
    ADD CONSTRAINT github_repositories_pkey PRIMARY KEY (id);


--
-- Name: github_repository_syncs github_repository_syncs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repository_syncs
    ADD CONSTRAINT github_repository_syncs_pkey PRIMARY KEY (id);


--
-- Name: github_repository_syncs github_repository_syncs_repository_id_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repository_syncs
    ADD CONSTRAINT github_repository_syncs_repository_id_key UNIQUE (repository_id);


--
-- Name: gitlab_comment_syncs gitlab_comment_syncs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_comment_syncs
    ADD CONSTRAINT gitlab_comment_syncs_pkey PRIMARY KEY (id);


--
-- Name: gitlab_issue_syncs gitlab_issue_syncs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_issue_syncs
    ADD CONSTRAINT gitlab_issue_syncs_pkey PRIMARY KEY (id);


--
-- Name: gitlab_repositories gitlab_repositories_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repositories
    ADD CONSTRAINT gitlab_repositories_pkey PRIMARY KEY (id);


--
-- Name: gitlab_repository_syncs gitlab_repository_syncs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repository_syncs
    ADD CONSTRAINT gitlab_repository_syncs_pkey PRIMARY KEY (id);


--
-- Name: gitlab_repository_syncs gitlab_repository_syncs_repository_id_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repository_syncs
    ADD CONSTRAINT gitlab_repository_syncs_repository_id_key UNIQUE (repository_id);


--
-- Name: importers importers_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.importers
    ADD CONSTRAINT importers_pkey PRIMARY KEY (id);


--
-- Name: instance_admins instance_admins_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instance_admins
    ADD CONSTRAINT instance_admins_pkey PRIMARY KEY (id);


--
-- Name: instance_configurations instance_configurations_key_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instance_configurations
    ADD CONSTRAINT instance_configurations_key_key UNIQUE (key);


--
-- Name: instance_configurations instance_configurations_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instance_configurations
    ADD CONSTRAINT instance_configurations_pkey PRIMARY KEY (id);


--
-- Name: instances instances_instance_id_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instances
    ADD CONSTRAINT instances_instance_id_key UNIQUE (instance_id);


--
-- Name: instances instances_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instances
    ADD CONSTRAINT instances_pkey PRIMARY KEY (id);


--
-- Name: intake_issues intake_issues_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intake_issues
    ADD CONSTRAINT intake_issues_pkey PRIMARY KEY (id);


--
-- Name: intakes intakes_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intakes
    ADD CONSTRAINT intakes_pkey PRIMARY KEY (id);


--
-- Name: integrations integrations_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.integrations
    ADD CONSTRAINT integrations_pkey PRIMARY KEY (id);


--
-- Name: integrations integrations_provider_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.integrations
    ADD CONSTRAINT integrations_provider_key UNIQUE (provider);


--
-- Name: issue_activities issue_activities_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_activities
    ADD CONSTRAINT issue_activities_pkey PRIMARY KEY (id);


--
-- Name: issue_assignees issue_assignees_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_assignees
    ADD CONSTRAINT issue_assignees_pkey PRIMARY KEY (id);


--
-- Name: issue_attachments issue_attachments_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_attachments
    ADD CONSTRAINT issue_attachments_pkey PRIMARY KEY (id);


--
-- Name: issue_blockers issue_blockers_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_blockers
    ADD CONSTRAINT issue_blockers_pkey PRIMARY KEY (id);


--
-- Name: issue_comments issue_comments_description_id_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_comments
    ADD CONSTRAINT issue_comments_description_id_key UNIQUE (description_id);


--
-- Name: issue_comments issue_comments_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_comments
    ADD CONSTRAINT issue_comments_pkey PRIMARY KEY (id);


--
-- Name: issue_description_versions issue_description_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_description_versions
    ADD CONSTRAINT issue_description_versions_pkey PRIMARY KEY (id);


--
-- Name: issue_labels issue_labels_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_labels
    ADD CONSTRAINT issue_labels_pkey PRIMARY KEY (id);


--
-- Name: issue_links issue_links_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_links
    ADD CONSTRAINT issue_links_pkey PRIMARY KEY (id);


--
-- Name: issue_mentions issue_mentions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_mentions
    ADD CONSTRAINT issue_mentions_pkey PRIMARY KEY (id);


--
-- Name: issue_reactions issue_reactions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_reactions
    ADD CONSTRAINT issue_reactions_pkey PRIMARY KEY (id);


--
-- Name: issue_relations issue_relations_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_relations
    ADD CONSTRAINT issue_relations_pkey PRIMARY KEY (id);


--
-- Name: issue_sequences issue_sequences_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_sequences
    ADD CONSTRAINT issue_sequences_pkey PRIMARY KEY (id);


--
-- Name: issue_subscribers issue_subscribers_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_subscribers
    ADD CONSTRAINT issue_subscribers_pkey PRIMARY KEY (id);


--
-- Name: issue_types issue_types_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_types
    ADD CONSTRAINT issue_types_pkey PRIMARY KEY (id);


--
-- Name: issue_versions issue_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_versions
    ADD CONSTRAINT issue_versions_pkey PRIMARY KEY (id);


--
-- Name: issue_views issue_views_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_views
    ADD CONSTRAINT issue_views_pkey PRIMARY KEY (id);


--
-- Name: issue_votes issue_votes_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_votes
    ADD CONSTRAINT issue_votes_pkey PRIMARY KEY (id);


--
-- Name: issues issues_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issues
    ADD CONSTRAINT issues_pkey PRIMARY KEY (id);


--
-- Name: labels labels_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.labels
    ADD CONSTRAINT labels_pkey PRIMARY KEY (id);


--
-- Name: module_issues module_issues_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_issues
    ADD CONSTRAINT module_issues_pkey PRIMARY KEY (id);


--
-- Name: module_links module_links_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_links
    ADD CONSTRAINT module_links_pkey PRIMARY KEY (id);


--
-- Name: module_members module_members_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_members
    ADD CONSTRAINT module_members_pkey PRIMARY KEY (id);


--
-- Name: module_user_properties module_user_properties_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_user_properties
    ADD CONSTRAINT module_user_properties_pkey PRIMARY KEY (id);


--
-- Name: modules modules_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.modules
    ADD CONSTRAINT modules_pkey PRIMARY KEY (id);


--
-- Name: notifications notifications_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_pkey PRIMARY KEY (id);


--
-- Name: page_labels page_labels_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_labels
    ADD CONSTRAINT page_labels_pkey PRIMARY KEY (id);


--
-- Name: page_logs page_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_logs
    ADD CONSTRAINT page_logs_pkey PRIMARY KEY (id);


--
-- Name: page_versions page_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_versions
    ADD CONSTRAINT page_versions_pkey PRIMARY KEY (id);


--
-- Name: pages pages_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_pkey PRIMARY KEY (id);


--
-- Name: profiles profiles_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.profiles
    ADD CONSTRAINT profiles_pkey PRIMARY KEY (id);


--
-- Name: profiles profiles_user_id_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.profiles
    ADD CONSTRAINT profiles_user_id_key UNIQUE (user_id);


--
-- Name: project_deploy_boards project_deploy_boards_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_deploy_boards
    ADD CONSTRAINT project_deploy_boards_pkey PRIMARY KEY (id);


--
-- Name: project_identifiers project_identifiers_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_identifiers
    ADD CONSTRAINT project_identifiers_pkey PRIMARY KEY (id);


--
-- Name: project_identifiers project_identifiers_project_id_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_identifiers
    ADD CONSTRAINT project_identifiers_project_id_key UNIQUE (project_id);


--
-- Name: project_issue_types project_issue_types_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_issue_types
    ADD CONSTRAINT project_issue_types_pkey PRIMARY KEY (id);


--
-- Name: project_member_invites project_member_invites_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_member_invites
    ADD CONSTRAINT project_member_invites_pkey PRIMARY KEY (id);


--
-- Name: project_members project_members_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_members_pkey PRIMARY KEY (id);


--
-- Name: project_pages project_pages_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_pages
    ADD CONSTRAINT project_pages_pkey PRIMARY KEY (id);


--
-- Name: project_public_members project_public_members_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_public_members
    ADD CONSTRAINT project_public_members_pkey PRIMARY KEY (id);


--
-- Name: project_user_properties project_user_properties_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_user_properties
    ADD CONSTRAINT project_user_properties_pkey PRIMARY KEY (id);


--
-- Name: project_webhooks project_webhooks_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_webhooks
    ADD CONSTRAINT project_webhooks_pkey PRIMARY KEY (id);


--
-- Name: projects projects_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_pkey PRIMARY KEY (id);


--
-- Name: seaql_migrations seaql_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.seaql_migrations
    ADD CONSTRAINT seaql_migrations_pkey PRIMARY KEY (version);


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (session_key);


--
-- Name: slack_project_syncs slack_project_syncs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.slack_project_syncs
    ADD CONSTRAINT slack_project_syncs_pkey PRIMARY KEY (id);


--
-- Name: social_login_connections social_login_connections_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.social_login_connections
    ADD CONSTRAINT social_login_connections_pkey PRIMARY KEY (id);


--
-- Name: states states_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.states
    ADD CONSTRAINT states_pkey PRIMARY KEY (id);


--
-- Name: stickies stickies_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.stickies
    ADD CONSTRAINT stickies_pkey PRIMARY KEY (id);


--
-- Name: teams teams_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.teams
    ADD CONSTRAINT teams_pkey PRIMARY KEY (id);


--
-- Name: user_favorites user_favorites_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_favorites
    ADD CONSTRAINT user_favorites_pkey PRIMARY KEY (id);


--
-- Name: user_github_connections user_github_connections_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_github_connections
    ADD CONSTRAINT user_github_connections_pkey PRIMARY KEY (id);


--
-- Name: user_github_connections user_github_connections_user_id_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_github_connections
    ADD CONSTRAINT user_github_connections_user_id_key UNIQUE (user_id);


--
-- Name: user_notification_preferences user_notification_preferences_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_notification_preferences
    ADD CONSTRAINT user_notification_preferences_pkey PRIMARY KEY (id);


--
-- Name: user_recent_visits user_recent_visits_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_recent_visits
    ADD CONSTRAINT user_recent_visits_pkey PRIMARY KEY (id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users_groups users_groups_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users_groups
    ADD CONSTRAINT users_groups_pkey PRIMARY KEY (id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users_user_permissions users_user_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users_user_permissions
    ADD CONSTRAINT users_user_permissions_pkey PRIMARY KEY (id);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- Name: webhook_logs webhook_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.webhook_logs
    ADD CONSTRAINT webhook_logs_pkey PRIMARY KEY (id);


--
-- Name: webhooks webhooks_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.webhooks
    ADD CONSTRAINT webhooks_pkey PRIMARY KEY (id);


--
-- Name: workspace_home_preferences workspace_home_preferences_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_home_preferences
    ADD CONSTRAINT workspace_home_preferences_pkey PRIMARY KEY (id);


--
-- Name: workspace_integrations workspace_integrations_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_integrations
    ADD CONSTRAINT workspace_integrations_pkey PRIMARY KEY (id);


--
-- Name: workspace_member_invites workspace_member_invites_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_member_invites
    ADD CONSTRAINT workspace_member_invites_pkey PRIMARY KEY (id);


--
-- Name: workspace_members workspace_members_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_members_pkey PRIMARY KEY (id);


--
-- Name: workspace_themes workspace_themes_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_themes
    ADD CONSTRAINT workspace_themes_pkey PRIMARY KEY (id);


--
-- Name: workspace_user_links workspace_user_links_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_links
    ADD CONSTRAINT workspace_user_links_pkey PRIMARY KEY (id);


--
-- Name: workspace_user_preferences workspace_user_preferences_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_preferences
    ADD CONSTRAINT workspace_user_preferences_pkey PRIMARY KEY (id);


--
-- Name: workspace_user_properties workspace_user_properties_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_properties
    ADD CONSTRAINT workspace_user_properties_pkey PRIMARY KEY (id);


--
-- Name: workspaces workspaces_pkey; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspaces
    ADD CONSTRAINT workspaces_pkey PRIMARY KEY (id);


--
-- Name: workspaces workspaces_slug_key; Type: CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspaces
    ADD CONSTRAINT workspaces_slug_key UNIQUE (slug);


--
-- Name: accounts_provider_provider_account_id_daac1f10_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX accounts_provider_provider_account_id_daac1f10_uniq ON public.accounts USING btree (provider, provider_account_id);


--
-- Name: accounts_user_id_7f1e1f1e; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX accounts_user_id_7f1e1f1e ON public.accounts USING btree (user_id);


--
-- Name: analytic_views_created_by_id_1b3ca0a9; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX analytic_views_created_by_id_1b3ca0a9 ON public.analytic_views USING btree (created_by_id);


--
-- Name: analytic_views_updated_by_id_b6d827e1; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX analytic_views_updated_by_id_b6d827e1 ON public.analytic_views USING btree (updated_by_id);


--
-- Name: analytic_views_workspace_id_ca6e5c0b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX analytic_views_workspace_id_ca6e5c0b ON public.analytic_views USING btree (workspace_id);


--
-- Name: api_tokens_user_id_2db24e1c; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX api_tokens_user_id_2db24e1c ON public.api_tokens USING btree (user_id);


--
-- Name: api_tokens_workspace_id_6791c7bd; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX api_tokens_workspace_id_6791c7bd ON public.api_tokens USING btree (workspace_id);


--
-- Name: asset_asset_idx; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX asset_asset_idx ON public.file_assets USING btree (asset);


--
-- Name: asset_entity_identifier_idx; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX asset_entity_identifier_idx ON public.file_assets USING btree (entity_identifier);


--
-- Name: asset_entity_idx; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX asset_entity_idx ON public.file_assets USING btree (entity_type, entity_identifier);


--
-- Name: asset_entity_type_idx; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX asset_entity_type_idx ON public.file_assets USING btree (entity_type);


--
-- Name: auth_group_permissions_group_id_b120cbf9; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX auth_group_permissions_group_id_b120cbf9 ON public.auth_group_permissions USING btree (group_id);


--
-- Name: auth_group_permissions_group_id_permission_id_0cd325b0_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX auth_group_permissions_group_id_permission_id_0cd325b0_uniq ON public.auth_group_permissions USING btree (group_id, permission_id);


--
-- Name: auth_group_permissions_permission_id_84c5c92e; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX auth_group_permissions_permission_id_84c5c92e ON public.auth_group_permissions USING btree (permission_id);


--
-- Name: auth_permission_content_type_id_2f476e4b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX auth_permission_content_type_id_2f476e4b ON public.auth_permission USING btree (content_type_id);


--
-- Name: auth_permission_content_type_id_codename_01ab375a_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX auth_permission_content_type_id_codename_01ab375a_uniq ON public.auth_permission USING btree (content_type_id, codename);


--
-- Name: comment_reactions_comment_id_actor_id_reac_24dc2de6_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX comment_reactions_comment_id_actor_id_reac_24dc2de6_uniq ON public.comment_reactions USING btree (comment_id, actor_id, reaction, deleted_at);


--
-- Name: cycle_issues_issue_id_cycle_id_deleted_at_93e8fecd_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX cycle_issues_issue_id_cycle_id_deleted_at_93e8fecd_uniq ON public.cycle_issues USING btree (issue_id, cycle_id, deleted_at);


--
-- Name: cycle_user_properties_cycle_id_user_id_deleted_at_fbe00cf4_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX cycle_user_properties_cycle_id_user_id_deleted_at_fbe00cf4_uniq ON public.cycle_user_properties USING btree (cycle_id, user_id, deleted_at);


--
-- Name: db_githubprstatemapping_workspace_integration_id_8a615667_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX db_githubprstatemapping_workspace_integration_id_8a615667_uniq ON public.db_githubprstatemapping USING btree (workspace_integration_id, project_id, github_pr_state);


--
-- Name: deploy_boards_entity_name_entity_ident_800ce160_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX deploy_boards_entity_name_entity_ident_800ce160_uniq ON public.deploy_boards USING btree (entity_name, entity_identifier, deleted_at);


--
-- Name: device_sessions_device_id_a42b2ada; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX device_sessions_device_id_a42b2ada ON public.device_sessions USING btree (device_id);


--
-- Name: device_sessions_session_id_5382b02b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX device_sessions_session_id_5382b02b ON public.device_sessions USING btree (session_id);


--
-- Name: devices_user_id_9a5cca49; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX devices_user_id_9a5cca49 ON public.devices USING btree (user_id);


--
-- Name: django_celery_beat_periodictask_clocked_id_47a69f82; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX django_celery_beat_periodictask_clocked_id_47a69f82 ON public.django_celery_beat_periodictask USING btree (clocked_id);


--
-- Name: django_celery_beat_periodictask_crontab_id_d3cba168; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX django_celery_beat_periodictask_crontab_id_d3cba168 ON public.django_celery_beat_periodictask USING btree (crontab_id);


--
-- Name: django_celery_beat_periodictask_interval_id_a8ca27da; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX django_celery_beat_periodictask_interval_id_a8ca27da ON public.django_celery_beat_periodictask USING btree (interval_id);


--
-- Name: django_celery_beat_periodictask_solar_id_a87ce72c; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX django_celery_beat_periodictask_solar_id_a87ce72c ON public.django_celery_beat_periodictask USING btree (solar_id);


--
-- Name: django_celery_beat_solar_event_latitude_longitude_ba64999a_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX django_celery_beat_solar_event_latitude_longitude_ba64999a_uniq ON public.django_celery_beat_solarschedule USING btree (event, latitude, longitude);


--
-- Name: django_content_type_app_label_model_76bd3d3b_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX django_content_type_app_label_model_76bd3d3b_uniq ON public.django_content_type USING btree (app_label, model);


--
-- Name: django_session_expire_date_a5c62663; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX django_session_expire_date_a5c62663 ON public.django_session USING btree (expire_date);


--
-- Name: draft_issue_assignee_unique_issue_assignee_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX draft_issue_assignee_unique_issue_assignee_when_deleted_at_null ON public.draft_issue_assignees USING btree (draft_issue_id, assignee_id) WHERE (deleted_at IS NULL);


--
-- Name: draft_issue_assignees_assignee_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_assignees_assignee_id ON public.draft_issue_assignees USING btree (assignee_id);


--
-- Name: draft_issue_assignees_created_by_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_assignees_created_by_id ON public.draft_issue_assignees USING btree (created_by_id);


--
-- Name: draft_issue_assignees_draft_issue_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_assignees_draft_issue_id ON public.draft_issue_assignees USING btree (draft_issue_id);


--
-- Name: draft_issue_assignees_project_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_assignees_project_id ON public.draft_issue_assignees USING btree (project_id);


--
-- Name: draft_issue_assignees_updated_by_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_assignees_updated_by_id ON public.draft_issue_assignees USING btree (updated_by_id);


--
-- Name: draft_issue_assignees_workspace_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_assignees_workspace_id ON public.draft_issue_assignees USING btree (workspace_id);


--
-- Name: draft_issue_cycle_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX draft_issue_cycle_when_deleted_at_null ON public.draft_issue_cycles USING btree (draft_issue_id, cycle_id) WHERE (deleted_at IS NULL);


--
-- Name: draft_issue_cycles_created_by_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_cycles_created_by_id ON public.draft_issue_cycles USING btree (created_by_id);


--
-- Name: draft_issue_cycles_cycle_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_cycles_cycle_id ON public.draft_issue_cycles USING btree (cycle_id);


--
-- Name: draft_issue_cycles_draft_issue_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_cycles_draft_issue_id ON public.draft_issue_cycles USING btree (draft_issue_id);


--
-- Name: draft_issue_cycles_project_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_cycles_project_id ON public.draft_issue_cycles USING btree (project_id);


--
-- Name: draft_issue_cycles_updated_by_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_cycles_updated_by_id ON public.draft_issue_cycles USING btree (updated_by_id);


--
-- Name: draft_issue_cycles_workspace_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_cycles_workspace_id ON public.draft_issue_cycles USING btree (workspace_id);


--
-- Name: draft_issue_labels_created_by_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_labels_created_by_id ON public.draft_issue_labels USING btree (created_by_id);


--
-- Name: draft_issue_labels_draft_issue_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_labels_draft_issue_id ON public.draft_issue_labels USING btree (draft_issue_id);


--
-- Name: draft_issue_labels_label_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_labels_label_id ON public.draft_issue_labels USING btree (label_id);


--
-- Name: draft_issue_labels_project_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_labels_project_id ON public.draft_issue_labels USING btree (project_id);


--
-- Name: draft_issue_labels_updated_by_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_labels_updated_by_id ON public.draft_issue_labels USING btree (updated_by_id);


--
-- Name: draft_issue_labels_workspace_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_labels_workspace_id ON public.draft_issue_labels USING btree (workspace_id);


--
-- Name: draft_issue_modules_created_by_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_modules_created_by_id ON public.draft_issue_modules USING btree (created_by_id);


--
-- Name: draft_issue_modules_draft_issue_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_modules_draft_issue_id ON public.draft_issue_modules USING btree (draft_issue_id);


--
-- Name: draft_issue_modules_module_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_modules_module_id ON public.draft_issue_modules USING btree (module_id);


--
-- Name: draft_issue_modules_project_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_modules_project_id ON public.draft_issue_modules USING btree (project_id);


--
-- Name: draft_issue_modules_updated_by_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_modules_updated_by_id ON public.draft_issue_modules USING btree (updated_by_id);


--
-- Name: draft_issue_modules_workspace_id; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX draft_issue_modules_workspace_id ON public.draft_issue_modules USING btree (workspace_id);


--
-- Name: estimate_points_estimate_id_4b4cb706; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX estimate_points_estimate_id_4b4cb706 ON public.estimate_points USING btree (estimate_id);


--
-- Name: estimate_points_project_id_ba9bcb2c; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX estimate_points_project_id_ba9bcb2c ON public.estimate_points USING btree (project_id);


--
-- Name: estimate_unique_name_project_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX estimate_unique_name_project_when_deleted_at_null ON public.estimates USING btree (name, project_id) WHERE (deleted_at IS NULL);


--
-- Name: estimates_project_id_7f195a41; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX estimates_project_id_7f195a41 ON public.estimates USING btree (project_id);


--
-- Name: file_asset_created_by_id_966942a0; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX file_asset_created_by_id_966942a0 ON public.file_assets USING btree (created_by_id);


--
-- Name: file_asset_updated_by_id_d6aaf4f0; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX file_asset_updated_by_id_d6aaf4f0 ON public.file_assets USING btree (updated_by_id);


--
-- Name: file_assets_user_id_ce1818dc; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX file_assets_user_id_ce1818dc ON public.file_assets USING btree (user_id);


--
-- Name: file_assets_workspace_id_fa50b9c5; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX file_assets_workspace_id_fa50b9c5 ON public.file_assets USING btree (workspace_id);


--
-- Name: github_comment_syncs_issue_sync_id_comment_id_38c82e7b_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX github_comment_syncs_issue_sync_id_comment_id_38c82e7b_uniq ON public.github_comment_syncs USING btree (issue_sync_id, comment_id);


--
-- Name: github_issue_syncs_repository_sync_id_issue_id_4b34427e_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX github_issue_syncs_repository_sync_id_issue_id_4b34427e_uniq ON public.github_issue_syncs USING btree (repository_sync_id, issue_id);


--
-- Name: github_repository_syncs_project_id_repository_id_0f3705e6_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX github_repository_syncs_project_id_repository_id_0f3705e6_uniq ON public.github_repository_syncs USING btree (project_id, repository_id);


--
-- Name: gitlab_comment_syncs_issue_sync_id_comment_id_61435f60_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX gitlab_comment_syncs_issue_sync_id_comment_id_61435f60_uniq ON public.gitlab_comment_syncs USING btree (issue_sync_id, comment_id);


--
-- Name: gitlab_issue_syncs_repository_sync_id_issue_id_14bdcc2c_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX gitlab_issue_syncs_repository_sync_id_issue_id_14bdcc2c_uniq ON public.gitlab_issue_syncs USING btree (repository_sync_id, issue_id);


--
-- Name: gitlab_repository_syncs_project_id_repository_id_13a57d00_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX gitlab_repository_syncs_project_id_repository_id_13a57d00_uniq ON public.gitlab_repository_syncs USING btree (project_id, repository_id);


--
-- Name: inboxes_name_project_id_deleted_at_95043f72_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX inboxes_name_project_id_deleted_at_95043f72_uniq ON public.intakes USING btree (name, project_id, deleted_at);


--
-- Name: instance_admins_instance_id_user_id_2e80a466_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX instance_admins_instance_id_user_id_2e80a466_uniq ON public.instance_admins USING btree (instance_id, user_id);


--
-- Name: issue_activity_actor_id_52fdd42d; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_activity_actor_id_52fdd42d ON public.issue_activities USING btree (actor_id);


--
-- Name: issue_activity_created_by_id_49516e3d; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_activity_created_by_id_49516e3d ON public.issue_activities USING btree (created_by_id);


--
-- Name: issue_activity_issue_comment_id_701f3c3c; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_activity_issue_comment_id_701f3c3c ON public.issue_activities USING btree (issue_comment_id);


--
-- Name: issue_activity_issue_id_807fbde4; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_activity_issue_id_807fbde4 ON public.issue_activities USING btree (issue_id);


--
-- Name: issue_activity_project_id_d0ac2ccf; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_activity_project_id_d0ac2ccf ON public.issue_activities USING btree (project_id);


--
-- Name: issue_activity_updated_by_id_0075f9bd; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_activity_updated_by_id_0075f9bd ON public.issue_activities USING btree (updated_by_id);


--
-- Name: issue_activity_workspace_id_65acaf73; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_activity_workspace_id_65acaf73 ON public.issue_activities USING btree (workspace_id);


--
-- Name: issue_assignee_assignee_id_50f5c04e; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_assignee_assignee_id_50f5c04e ON public.issue_assignees USING btree (assignee_id);


--
-- Name: issue_assignee_created_by_id_f693d43b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_assignee_created_by_id_f693d43b ON public.issue_assignees USING btree (created_by_id);


--
-- Name: issue_assignee_issue_id_72da08db; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_assignee_issue_id_72da08db ON public.issue_assignees USING btree (issue_id);


--
-- Name: issue_assignee_project_id_61c18bf2; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_assignee_project_id_61c18bf2 ON public.issue_assignees USING btree (project_id);


--
-- Name: issue_assignee_unique_issue_assignee_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX issue_assignee_unique_issue_assignee_when_deleted_at_null ON public.issue_assignees USING btree (issue_id, assignee_id);


--
-- Name: issue_assignee_updated_by_id_c54088aa; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_assignee_updated_by_id_c54088aa ON public.issue_assignees USING btree (updated_by_id);


--
-- Name: issue_assignee_workspace_id_9aad55b7; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_assignee_workspace_id_9aad55b7 ON public.issue_assignees USING btree (workspace_id);


--
-- Name: issue_assignees_issue_id_assignee_id_deleted_at_b2623a0e_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX issue_assignees_issue_id_assignee_id_deleted_at_b2623a0e_uniq ON public.issue_assignees USING btree (issue_id, assignee_id, deleted_at);


--
-- Name: issue_comment_actor_id_d312315b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_comment_actor_id_d312315b ON public.issue_comments USING btree (actor_id);


--
-- Name: issue_comment_created_by_id_0765f239; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_comment_created_by_id_0765f239 ON public.issue_comments USING btree (created_by_id);


--
-- Name: issue_comment_issue_id_d0195e35; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_comment_issue_id_d0195e35 ON public.issue_comments USING btree (issue_id);


--
-- Name: issue_comment_project_id_db37c105; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_comment_project_id_db37c105 ON public.issue_comments USING btree (project_id);


--
-- Name: issue_comment_updated_by_id_96cfb86e; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_comment_updated_by_id_96cfb86e ON public.issue_comments USING btree (updated_by_id);


--
-- Name: issue_comment_workspace_id_3f7969ec; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_comment_workspace_id_3f7969ec ON public.issue_comments USING btree (workspace_id);


--
-- Name: issue_comments_parent_id_d8db10b1; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_comments_parent_id_d8db10b1 ON public.issue_comments USING btree (parent_id);


--
-- Name: issue_created_by_id_8f0ae62b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_created_by_id_8f0ae62b ON public.issues USING btree (created_by_id);


--
-- Name: issue_parent_id_ce8d76ba; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_parent_id_ce8d76ba ON public.issues USING btree (parent_id);


--
-- Name: issue_project_id_fea0fc80; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_project_id_fea0fc80 ON public.issues USING btree (project_id);


--
-- Name: issue_property_project_id_30e7de7b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_property_project_id_30e7de7b ON public.project_user_properties USING btree (project_id);


--
-- Name: issue_property_user_id_0b1d1c8f; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_property_user_id_0b1d1c8f ON public.project_user_properties USING btree (user_id);


--
-- Name: issue_reactions_issue_id_actor_id_reacti_7da73ced_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX issue_reactions_issue_id_actor_id_reacti_7da73ced_uniq ON public.issue_reactions USING btree (issue_id, actor_id, reaction, deleted_at);


--
-- Name: issue_relations_issue_id_related_issue_i_cc724584_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX issue_relations_issue_id_related_issue_i_cc724584_uniq ON public.issue_relations USING btree (issue_id, related_issue_id, deleted_at);


--
-- Name: issue_state_id_1a65560d; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_state_id_1a65560d ON public.issues USING btree (state_id);


--
-- Name: issue_subscribers_issue_id_subscriber_id_d_587dec1a_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX issue_subscribers_issue_id_subscriber_id_d_587dec1a_uniq ON public.issue_subscribers USING btree (issue_id, subscriber_id, deleted_at);


--
-- Name: issue_types_workspace_id_591c6f3b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_types_workspace_id_591c6f3b ON public.issue_types USING btree (workspace_id);


--
-- Name: issue_updated_by_id_f1261863; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_updated_by_id_f1261863 ON public.issues USING btree (updated_by_id);


--
-- Name: issue_votes_issue_id_actor_id_deleted_at_886f34e8_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX issue_votes_issue_id_actor_id_deleted_at_886f34e8_uniq ON public.issue_votes USING btree (issue_id, actor_id, deleted_at);


--
-- Name: issue_workspace_id_c84878c1; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issue_workspace_id_c84878c1 ON public.issues USING btree (workspace_id);


--
-- Name: issues_estimate_point_id_a6822abe; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issues_estimate_point_id_a6822abe ON public.issues USING btree (estimate_point_id);


--
-- Name: issues_type_id_a4710b19; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX issues_type_id_a4710b19 ON public.issues USING btree (type_id);


--
-- Name: label_parent_id_7a853296; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX label_parent_id_7a853296 ON public.labels USING btree (parent_id);


--
-- Name: label_project_id_90e0f1a2; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX label_project_id_90e0f1a2 ON public.labels USING btree (project_id);


--
-- Name: label_workspace_id_c4c9ae5a; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX label_workspace_id_c4c9ae5a ON public.labels USING btree (workspace_id);


--
-- Name: module_draft_issue_unique_issue_module_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX module_draft_issue_unique_issue_module_when_deleted_at_null ON public.draft_issue_modules USING btree (draft_issue_id, module_id) WHERE (deleted_at IS NULL);


--
-- Name: module_issues_issue_id_module_id_deleted_at_f944f7c9_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX module_issues_issue_id_module_id_deleted_at_f944f7c9_uniq ON public.module_issues USING btree (issue_id, module_id, deleted_at);


--
-- Name: module_members_module_id_member_id_deleted_at_bb7a6f00_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX module_members_module_id_member_id_deleted_at_bb7a6f00_uniq ON public.module_members USING btree (module_id, member_id, deleted_at);


--
-- Name: module_user_properties_module_id_user_id_delete_3269582d_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX module_user_properties_module_id_user_id_delete_3269582d_uniq ON public.module_user_properties USING btree (module_id, user_id, deleted_at);


--
-- Name: page_logs_page_id_transaction_9ab05334_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX page_logs_page_id_transaction_9ab05334_uniq ON public.page_logs USING btree (page_id, transaction);


--
-- Name: project_deploy_boards_created_by_id_2ea72f98; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_deploy_boards_created_by_id_2ea72f98 ON public.project_deploy_boards USING btree (created_by_id);


--
-- Name: project_deploy_boards_inbox_id_a6a75525; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_deploy_boards_inbox_id_a6a75525 ON public.project_deploy_boards USING btree (intake_id);


--
-- Name: project_deploy_boards_project_id_49d887b2; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_deploy_boards_project_id_49d887b2 ON public.project_deploy_boards USING btree (project_id);


--
-- Name: project_deploy_boards_updated_by_id_290eb99e; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_deploy_boards_updated_by_id_290eb99e ON public.project_deploy_boards USING btree (updated_by_id);


--
-- Name: project_deploy_boards_workspace_id_cd92f164; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_deploy_boards_workspace_id_cd92f164 ON public.project_deploy_boards USING btree (workspace_id);


--
-- Name: project_identifiers_name_6ca8a4b0; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_identifiers_name_6ca8a4b0 ON public.project_identifiers USING btree (name);


--
-- Name: project_issue_type_unique_project_issue_type_when_deleted_at_nu; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX project_issue_type_unique_project_issue_type_when_deleted_at_nu ON public.project_issue_types USING btree (project_id, issue_type_id) WHERE (deleted_at IS NULL);


--
-- Name: project_issue_types_issue_type_id_9494de9f; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_issue_types_issue_type_id_9494de9f ON public.project_issue_types USING btree (issue_type_id);


--
-- Name: project_issue_types_project_id_ef6e52e4; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_issue_types_project_id_ef6e52e4 ON public.project_issue_types USING btree (project_id);


--
-- Name: project_member_invite_project_id_8fb7750e; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_member_invite_project_id_8fb7750e ON public.project_member_invites USING btree (project_id);


--
-- Name: project_member_member_id_9d6b126b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_member_member_id_9d6b126b ON public.project_members USING btree (member_id);


--
-- Name: project_member_project_id_11ea1a9e; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_member_project_id_11ea1a9e ON public.project_members USING btree (project_id);


--
-- Name: project_member_unique_project_member_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX project_member_unique_project_member_when_deleted_at_null ON public.project_members USING btree (project_id, member_id) WHERE (deleted_at IS NULL);


--
-- Name: project_pages_project_id_page_id_deleted_at_7c80a40c_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX project_pages_project_id_page_id_deleted_at_7c80a40c_uniq ON public.project_pages USING btree (project_id, page_id, deleted_at);


--
-- Name: project_public_member_unique_project_member_when_deleted_at_nul; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX project_public_member_unique_project_member_when_deleted_at_nul ON public.project_public_members USING btree (project_id, member_id) WHERE (deleted_at IS NULL);


--
-- Name: project_unique_identifier_workspace_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX project_unique_identifier_workspace_when_deleted_at_null ON public.projects USING btree (identifier, workspace_id) WHERE (deleted_at IS NULL);


--
-- Name: project_unique_name_workspace_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX project_unique_name_workspace_when_deleted_at_null ON public.projects USING btree (name, workspace_id) WHERE (deleted_at IS NULL);


--
-- Name: project_user_property_unique_user_project_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX project_user_property_unique_user_project_when_deleted_at_null ON public.project_user_properties USING btree (user_id, project_id) WHERE (deleted_at IS NULL);


--
-- Name: project_webhook_unique_project_webhook_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX project_webhook_unique_project_webhook_when_deleted_at_null ON public.project_webhooks USING btree (project_id, webhook_id) WHERE (deleted_at IS NULL);


--
-- Name: project_webhooks_project_id_bec3cf8c; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_webhooks_project_id_bec3cf8c ON public.project_webhooks USING btree (project_id);


--
-- Name: project_webhooks_webhook_id_da27c6a7; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_webhooks_webhook_id_da27c6a7 ON public.project_webhooks USING btree (webhook_id);


--
-- Name: project_workspace_id_01764ff9; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX project_workspace_id_01764ff9 ON public.projects USING btree (workspace_id);


--
-- Name: projects_cover_image_asset_id_e6636b92; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX projects_cover_image_asset_id_e6636b92 ON public.projects USING btree (cover_image_asset_id);


--
-- Name: projects_default_state_id_f13e8b95; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX projects_default_state_id_f13e8b95 ON public.projects USING btree (default_state_id);


--
-- Name: projects_estimate_id_85c7b2ac; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX projects_estimate_id_85c7b2ac ON public.projects USING btree (estimate_id);


--
-- Name: projects_identifier_3267ade8; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX projects_identifier_3267ade8 ON public.projects USING btree (identifier);


--
-- Name: sessions_expire_date_16e4c444; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX sessions_expire_date_16e4c444 ON public.sessions USING btree (expire_date);


--
-- Name: sessions_user_id_05e26f4a; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX sessions_user_id_05e26f4a ON public.sessions USING btree (user_id);


--
-- Name: slack_project_syncs_team_id_project_id_50a144a7_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX slack_project_syncs_team_id_project_id_50a144a7_uniq ON public.slack_project_syncs USING btree (team_id, project_id);


--
-- Name: state_project_id_23a65fd6; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX state_project_id_23a65fd6 ON public.states USING btree (project_id);


--
-- Name: state_slug_bab0af35; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX state_slug_bab0af35 ON public.states USING btree (slug);


--
-- Name: state_unique_name_project_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX state_unique_name_project_when_deleted_at_null ON public.states USING btree (name, project_id) WHERE (deleted_at IS NULL);


--
-- Name: state_workspace_id_2293282d; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX state_workspace_id_2293282d ON public.states USING btree (workspace_id);


--
-- Name: teams_name_workspace_id_deleted_at_4b131aa2_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX teams_name_workspace_id_deleted_at_4b131aa2_uniq ON public.teams USING btree (name, workspace_id, deleted_at);


--
-- Name: unique_name_when_project_null_and_not_deleted; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX unique_name_when_project_null_and_not_deleted ON public.labels USING btree (name) WHERE ((deleted_at IS NULL) AND (project_id IS NULL));


--
-- Name: unique_name_workspace_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX unique_name_workspace_when_deleted_at_null ON public.project_identifiers USING btree (name, workspace_id) WHERE (deleted_at IS NULL);


--
-- Name: unique_project_name_when_not_deleted; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX unique_project_name_when_not_deleted ON public.labels USING btree (project_id, name) WHERE ((deleted_at IS NULL) AND (project_id IS NOT NULL));


--
-- Name: user_favorites_entity_type_user_id_enti_22b103ff_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX user_favorites_entity_type_user_id_enti_22b103ff_uniq ON public.user_favorites USING btree (entity_type, user_id, entity_identifier, deleted_at);


--
-- Name: user_groups_group_id_b76f8aba; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX user_groups_group_id_b76f8aba ON public.users_groups USING btree (group_id);


--
-- Name: user_groups_user_id_abaea130; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX user_groups_user_id_abaea130 ON public.users_groups USING btree (user_id);


--
-- Name: user_groups_user_id_group_id_40beef00_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX user_groups_user_id_group_id_40beef00_uniq ON public.users_groups USING btree (user_id, group_id);


--
-- Name: user_user_permissions_user_id_permission_id_7dc6e2e0_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX user_user_permissions_user_id_permission_id_7dc6e2e0_uniq ON public.users_user_permissions USING btree (user_id, permission_id);


--
-- Name: users_avatar_asset_id_50fa2043; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX users_avatar_asset_id_50fa2043 ON public.users USING btree (avatar_asset_id);


--
-- Name: users_cover_image_asset_id_b9679cbc; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX users_cover_image_asset_id_b9679cbc ON public.users USING btree (cover_image_asset_id);


--
-- Name: webhook_logs_workspace_id_ffcd0e31; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX webhook_logs_workspace_id_ffcd0e31 ON public.webhook_logs USING btree (workspace_id);


--
-- Name: webhook_url_unique_url_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX webhook_url_unique_url_when_deleted_at_null ON public.webhooks USING btree (workspace_id, url) WHERE (deleted_at IS NULL);


--
-- Name: webhooks_workspace_id_da5865d7; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX webhooks_workspace_id_da5865d7 ON public.webhooks USING btree (workspace_id);


--
-- Name: workspace_integrations_actor_id_21619aa1; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX workspace_integrations_actor_id_21619aa1 ON public.workspace_integrations USING btree (actor_id);


--
-- Name: workspace_integrations_api_token_id_bdb1759b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX workspace_integrations_api_token_id_bdb1759b ON public.workspace_integrations USING btree (api_token_id);


--
-- Name: workspace_integrations_integration_id_6cb0aace; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX workspace_integrations_integration_id_6cb0aace ON public.workspace_integrations USING btree (integration_id);


--
-- Name: workspace_integrations_workspace_id_27ebeb6b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX workspace_integrations_workspace_id_27ebeb6b ON public.workspace_integrations USING btree (workspace_id);


--
-- Name: workspace_integrations_workspace_id_integration_fa041c22_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX workspace_integrations_workspace_id_integration_fa041c22_uniq ON public.workspace_integrations USING btree (workspace_id, integration_id);


--
-- Name: workspace_member_invite_unique_email_workspace_when_deleted_at_; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX workspace_member_invite_unique_email_workspace_when_deleted_at_ ON public.workspace_member_invites USING btree (email, workspace_id) WHERE (deleted_at IS NULL);


--
-- Name: workspace_member_invites_email_workspace_id_delet_2f03573e_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX workspace_member_invites_email_workspace_id_delet_2f03573e_uniq ON public.workspace_member_invites USING btree (email, workspace_id, deleted_at);


--
-- Name: workspace_member_member_id_824f5497; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX workspace_member_member_id_824f5497 ON public.workspace_members USING btree (member_id);


--
-- Name: workspace_member_unique_workspace_member_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX workspace_member_unique_workspace_member_when_deleted_at_null ON public.workspace_members USING btree (workspace_id, member_id) WHERE (deleted_at IS NULL);


--
-- Name: workspace_member_workspace_id_33f66d4b; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX workspace_member_workspace_id_33f66d4b ON public.workspace_members USING btree (workspace_id);


--
-- Name: workspace_members_workspace_id_member_id_d_d7bfa872_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX workspace_members_workspace_id_member_id_d_d7bfa872_uniq ON public.workspace_members USING btree (workspace_id, member_id, deleted_at);


--
-- Name: workspace_owner_id_60a8bafc; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX workspace_owner_id_60a8bafc ON public.workspaces USING btree (owner_id);


--
-- Name: workspace_theme_unique_workspace_name_when_deleted_at_null; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX workspace_theme_unique_workspace_name_when_deleted_at_null ON public.workspace_themes USING btree (workspace_id, name) WHERE (deleted_at IS NULL);


--
-- Name: workspace_themes_workspace_id_name_deleted_at_b536ffd3_uniq; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX workspace_themes_workspace_id_name_deleted_at_b536ffd3_uniq ON public.workspace_themes USING btree (workspace_id, name, deleted_at);


--
-- Name: workspace_user_home_preferences_unique_workspace_user_key_when_; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX workspace_user_home_preferences_unique_workspace_user_key_when_ ON public.workspace_home_preferences USING btree (workspace_id, user_id, key) WHERE (deleted_at IS NULL);


--
-- Name: workspace_user_preferences_unique_workspace_user_key_when_delet; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX workspace_user_preferences_unique_workspace_user_key_when_delet ON public.workspace_user_preferences USING btree (workspace_id, user_id, key) WHERE (deleted_at IS NULL);


--
-- Name: workspace_user_properties_unique_workspace_user_when_deleted_at; Type: INDEX; Schema: public; Owner: plane
--

CREATE UNIQUE INDEX workspace_user_properties_unique_workspace_user_when_deleted_at ON public.workspace_user_properties USING btree (workspace_id, user_id) WHERE (deleted_at IS NULL);


--
-- Name: workspaces_logo_asset_id_a784bb00; Type: INDEX; Schema: public; Owner: plane
--

CREATE INDEX workspaces_logo_asset_id_a784bb00 ON public.workspaces USING btree (logo_asset_id);


--
-- Name: accounts accounts_user_id_7f1e1f1e_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.accounts
    ADD CONSTRAINT accounts_user_id_7f1e1f1e_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: analytic_views analytic_views_created_by_id_1b3ca0a9_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.analytic_views
    ADD CONSTRAINT analytic_views_created_by_id_1b3ca0a9_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: analytic_views analytic_views_updated_by_id_b6d827e1_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.analytic_views
    ADD CONSTRAINT analytic_views_updated_by_id_b6d827e1_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: analytic_views analytic_views_workspace_id_ca6e5c0b_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.analytic_views
    ADD CONSTRAINT analytic_views_workspace_id_ca6e5c0b_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: api_activity_logs api_activity_logs_created_by_id_7f5c4ca8_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.api_activity_logs
    ADD CONSTRAINT api_activity_logs_created_by_id_7f5c4ca8_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: api_activity_logs api_activity_logs_updated_by_id_9ba0d417_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.api_activity_logs
    ADD CONSTRAINT api_activity_logs_updated_by_id_9ba0d417_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: api_tokens api_tokens_created_by_id_441e3d24_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT api_tokens_created_by_id_441e3d24_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: api_tokens api_tokens_updated_by_id_bcd544cf_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT api_tokens_updated_by_id_bcd544cf_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: api_tokens api_tokens_user_id_2db24e1c_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT api_tokens_user_id_2db24e1c_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: api_tokens api_tokens_workspace_id_6791c7bd_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT api_tokens_workspace_id_6791c7bd_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: auth_group_permissions auth_group_permissio_permission_id_84c5c92e_fk_auth_perm; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.auth_group_permissions
    ADD CONSTRAINT auth_group_permissio_permission_id_84c5c92e_fk_auth_perm FOREIGN KEY (permission_id) REFERENCES public.auth_permission(id);


--
-- Name: auth_group_permissions auth_group_permissions_group_id_b120cbf9_fk_auth_group_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.auth_group_permissions
    ADD CONSTRAINT auth_group_permissions_group_id_b120cbf9_fk_auth_group_id FOREIGN KEY (group_id) REFERENCES public.auth_group(id);


--
-- Name: auth_permission auth_permission_content_type_id_2f476e4b_fk_django_co; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.auth_permission
    ADD CONSTRAINT auth_permission_content_type_id_2f476e4b_fk_django_co FOREIGN KEY (content_type_id) REFERENCES public.django_content_type(id);


--
-- Name: changelogs changelogs_created_by_id_16dd944a_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.changelogs
    ADD CONSTRAINT changelogs_created_by_id_16dd944a_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: changelogs changelogs_updated_by_id_e0989861_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.changelogs
    ADD CONSTRAINT changelogs_updated_by_id_e0989861_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: comment_reactions comment_reactions_actor_id_21219e9c_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_actor_id_21219e9c_fk_users_id FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: comment_reactions comment_reactions_comment_id_87c59446_fk_issue_comments_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_comment_id_87c59446_fk_issue_comments_id FOREIGN KEY (comment_id) REFERENCES public.issue_comments(id);


--
-- Name: comment_reactions comment_reactions_created_by_id_9aeb43c4_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_created_by_id_9aeb43c4_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: comment_reactions comment_reactions_project_id_ab9114b4_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_project_id_ab9114b4_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: comment_reactions comment_reactions_updated_by_id_c74c9bbd_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_updated_by_id_c74c9bbd_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: comment_reactions comment_reactions_workspace_id_b614ca4f_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.comment_reactions
    ADD CONSTRAINT comment_reactions_workspace_id_b614ca4f_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: cycles cycle_created_by_id_78e43b79_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycles
    ADD CONSTRAINT cycle_created_by_id_78e43b79_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: cycle_issues cycle_issue_created_by_id_30b27539_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_issues
    ADD CONSTRAINT cycle_issue_created_by_id_30b27539_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: cycle_issues cycle_issue_cycle_id_ec681215_fk_cycle_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_issues
    ADD CONSTRAINT cycle_issue_cycle_id_ec681215_fk_cycle_id FOREIGN KEY (cycle_id) REFERENCES public.cycles(id);


--
-- Name: cycle_issues cycle_issue_project_id_6ad3257a_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_issues
    ADD CONSTRAINT cycle_issue_project_id_6ad3257a_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: cycle_issues cycle_issue_updated_by_id_cb4516f2_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_issues
    ADD CONSTRAINT cycle_issue_updated_by_id_cb4516f2_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: cycle_issues cycle_issue_workspace_id_1d77330e_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_issues
    ADD CONSTRAINT cycle_issue_workspace_id_1d77330e_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: cycle_issues cycle_issues_issue_id_2d5ac97f_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_issues
    ADD CONSTRAINT cycle_issues_issue_id_2d5ac97f_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: cycles cycle_owned_by_id_5456a4d1_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycles
    ADD CONSTRAINT cycle_owned_by_id_5456a4d1_fk_user_id FOREIGN KEY (owned_by_id) REFERENCES public.users(id);


--
-- Name: cycles cycle_project_id_0b590349_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycles
    ADD CONSTRAINT cycle_project_id_0b590349_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: cycles cycle_updated_by_id_93baee43_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycles
    ADD CONSTRAINT cycle_updated_by_id_93baee43_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: cycle_user_properties cycle_user_properties_created_by_id_501f371c_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_user_properties
    ADD CONSTRAINT cycle_user_properties_created_by_id_501f371c_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: cycle_user_properties cycle_user_properties_cycle_id_1f8bdf35_fk_cycles_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_user_properties
    ADD CONSTRAINT cycle_user_properties_cycle_id_1f8bdf35_fk_cycles_id FOREIGN KEY (cycle_id) REFERENCES public.cycles(id);


--
-- Name: cycle_user_properties cycle_user_properties_project_id_4efc0f07_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_user_properties
    ADD CONSTRAINT cycle_user_properties_project_id_4efc0f07_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: cycle_user_properties cycle_user_properties_updated_by_id_1b5ac27b_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_user_properties
    ADD CONSTRAINT cycle_user_properties_updated_by_id_1b5ac27b_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: cycle_user_properties cycle_user_properties_user_id_9e9ef97d_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_user_properties
    ADD CONSTRAINT cycle_user_properties_user_id_9e9ef97d_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: cycle_user_properties cycle_user_properties_workspace_id_62d65d71_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycle_user_properties
    ADD CONSTRAINT cycle_user_properties_workspace_id_62d65d71_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: cycles cycle_workspace_id_a199e8e1_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.cycles
    ADD CONSTRAINT cycle_workspace_id_a199e8e1_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: db_githubprstatemapping db_githubprstatemapp_workspace_integratio_2eab555c_fk_workspace; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.db_githubprstatemapping
    ADD CONSTRAINT db_githubprstatemapp_workspace_integratio_2eab555c_fk_workspace FOREIGN KEY (workspace_integration_id) REFERENCES public.workspace_integrations(id);


--
-- Name: db_githubprstatemapping db_githubprstatemapping_created_by_id_381aa92b_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.db_githubprstatemapping
    ADD CONSTRAINT db_githubprstatemapping_created_by_id_381aa92b_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: db_githubprstatemapping db_githubprstatemapping_project_id_361c0ac2_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.db_githubprstatemapping
    ADD CONSTRAINT db_githubprstatemapping_project_id_361c0ac2_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: db_githubprstatemapping db_githubprstatemapping_state_id_d2959076_fk_states_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.db_githubprstatemapping
    ADD CONSTRAINT db_githubprstatemapping_state_id_d2959076_fk_states_id FOREIGN KEY (state_id) REFERENCES public.states(id);


--
-- Name: db_githubprstatemapping db_githubprstatemapping_updated_by_id_3330c9ff_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.db_githubprstatemapping
    ADD CONSTRAINT db_githubprstatemapping_updated_by_id_3330c9ff_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: deploy_boards deploy_boards_created_by_id_149dff93_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.deploy_boards
    ADD CONSTRAINT deploy_boards_created_by_id_149dff93_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: deploy_boards deploy_boards_intake_id_76a6470a_fk_intakes_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.deploy_boards
    ADD CONSTRAINT deploy_boards_intake_id_76a6470a_fk_intakes_id FOREIGN KEY (intake_id) REFERENCES public.intakes(id);


--
-- Name: deploy_boards deploy_boards_project_id_cfc792a1_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.deploy_boards
    ADD CONSTRAINT deploy_boards_project_id_cfc792a1_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: deploy_boards deploy_boards_updated_by_id_db7ae24f_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.deploy_boards
    ADD CONSTRAINT deploy_boards_updated_by_id_db7ae24f_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: deploy_boards deploy_boards_workspace_id_fcf03158_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.deploy_boards
    ADD CONSTRAINT deploy_boards_workspace_id_fcf03158_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: description_versions description_versions_created_by_id_6633a3de_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.description_versions
    ADD CONSTRAINT description_versions_created_by_id_6633a3de_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: description_versions description_versions_description_id_dc7f19b6_fk_descriptions_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.description_versions
    ADD CONSTRAINT description_versions_description_id_dc7f19b6_fk_descriptions_id FOREIGN KEY (description_id) REFERENCES public.descriptions(id);


--
-- Name: description_versions description_versions_project_id_1a6c9aa9_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.description_versions
    ADD CONSTRAINT description_versions_project_id_1a6c9aa9_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: description_versions description_versions_updated_by_id_8b5179ae_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.description_versions
    ADD CONSTRAINT description_versions_updated_by_id_8b5179ae_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: description_versions description_versions_workspace_id_52857186_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.description_versions
    ADD CONSTRAINT description_versions_workspace_id_52857186_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: descriptions descriptions_created_by_id_b88ab399_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.descriptions
    ADD CONSTRAINT descriptions_created_by_id_b88ab399_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: descriptions descriptions_project_id_8f46180b_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.descriptions
    ADD CONSTRAINT descriptions_project_id_8f46180b_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: descriptions descriptions_updated_by_id_af519c4d_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.descriptions
    ADD CONSTRAINT descriptions_updated_by_id_af519c4d_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: descriptions descriptions_workspace_id_767279bf_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.descriptions
    ADD CONSTRAINT descriptions_workspace_id_767279bf_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: device_sessions device_sessions_created_by_id_920a3bd5_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.device_sessions
    ADD CONSTRAINT device_sessions_created_by_id_920a3bd5_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: device_sessions device_sessions_device_id_a42b2ada_fk_devices_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.device_sessions
    ADD CONSTRAINT device_sessions_device_id_a42b2ada_fk_devices_id FOREIGN KEY (device_id) REFERENCES public.devices(id);


--
-- Name: device_sessions device_sessions_session_id_5382b02b_fk_sessions_session_key; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.device_sessions
    ADD CONSTRAINT device_sessions_session_id_5382b02b_fk_sessions_session_key FOREIGN KEY (session_id) REFERENCES public.sessions(session_key);


--
-- Name: device_sessions device_sessions_updated_by_id_d0bd0c76_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.device_sessions
    ADD CONSTRAINT device_sessions_updated_by_id_d0bd0c76_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: devices devices_created_by_id_410a755b_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_created_by_id_410a755b_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: devices devices_updated_by_id_ee20dc3c_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_updated_by_id_ee20dc3c_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: devices devices_user_id_9a5cca49_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.devices
    ADD CONSTRAINT devices_user_id_9a5cca49_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: django_celery_beat_periodictask django_celery_beat_p_clocked_id_47a69f82_fk_django_ce; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_periodictask
    ADD CONSTRAINT django_celery_beat_p_clocked_id_47a69f82_fk_django_ce FOREIGN KEY (clocked_id) REFERENCES public.django_celery_beat_clockedschedule(id);


--
-- Name: django_celery_beat_periodictask django_celery_beat_p_crontab_id_d3cba168_fk_django_ce; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_periodictask
    ADD CONSTRAINT django_celery_beat_p_crontab_id_d3cba168_fk_django_ce FOREIGN KEY (crontab_id) REFERENCES public.django_celery_beat_crontabschedule(id);


--
-- Name: django_celery_beat_periodictask django_celery_beat_p_interval_id_a8ca27da_fk_django_ce; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_periodictask
    ADD CONSTRAINT django_celery_beat_p_interval_id_a8ca27da_fk_django_ce FOREIGN KEY (interval_id) REFERENCES public.django_celery_beat_intervalschedule(id);


--
-- Name: django_celery_beat_periodictask django_celery_beat_p_solar_id_a87ce72c_fk_django_ce; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.django_celery_beat_periodictask
    ADD CONSTRAINT django_celery_beat_p_solar_id_a87ce72c_fk_django_ce FOREIGN KEY (solar_id) REFERENCES public.django_celery_beat_solarschedule(id);


--
-- Name: draft_issue_assignees draft_issue_assignees_assignee_id_9cc52f9d; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_assignees
    ADD CONSTRAINT draft_issue_assignees_assignee_id_9cc52f9d FOREIGN KEY (assignee_id) REFERENCES public.users(id);


--
-- Name: draft_issue_assignees draft_issue_assignees_created_by; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_assignees
    ADD CONSTRAINT draft_issue_assignees_created_by FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: draft_issue_assignees draft_issue_assignees_draft_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_assignees
    ADD CONSTRAINT draft_issue_assignees_draft_issue_id FOREIGN KEY (draft_issue_id) REFERENCES public.draft_issues(id);


--
-- Name: draft_issue_assignees draft_issue_assignees_updated_by; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_assignees
    ADD CONSTRAINT draft_issue_assignees_updated_by FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: draft_issue_assignees draft_issue_assignees_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_assignees
    ADD CONSTRAINT draft_issue_assignees_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: draft_issue_cycles draft_issue_cycles_created_by; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_cycles
    ADD CONSTRAINT draft_issue_cycles_created_by FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: draft_issue_cycles draft_issue_cycles_cycle_id_b214e11f; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_cycles
    ADD CONSTRAINT draft_issue_cycles_cycle_id_b214e11f FOREIGN KEY (cycle_id) REFERENCES public.cycles(id);


--
-- Name: draft_issue_cycles draft_issue_cycles_draft_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_cycles
    ADD CONSTRAINT draft_issue_cycles_draft_issue_id FOREIGN KEY (draft_issue_id) REFERENCES public.draft_issues(id);


--
-- Name: draft_issue_cycles draft_issue_cycles_updated_by; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_cycles
    ADD CONSTRAINT draft_issue_cycles_updated_by FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: draft_issue_cycles draft_issue_cycles_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_cycles
    ADD CONSTRAINT draft_issue_cycles_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: draft_issue_labels draft_issue_labels_created_by; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_labels
    ADD CONSTRAINT draft_issue_labels_created_by FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: draft_issue_labels draft_issue_labels_draft_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_labels
    ADD CONSTRAINT draft_issue_labels_draft_issue_id FOREIGN KEY (draft_issue_id) REFERENCES public.draft_issues(id);


--
-- Name: draft_issue_labels draft_issue_labels_label_id_b9b001a5; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_labels
    ADD CONSTRAINT draft_issue_labels_label_id_b9b001a5 FOREIGN KEY (label_id) REFERENCES public.labels(id);


--
-- Name: draft_issue_labels draft_issue_labels_updated_by; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_labels
    ADD CONSTRAINT draft_issue_labels_updated_by FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: draft_issue_labels draft_issue_labels_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_labels
    ADD CONSTRAINT draft_issue_labels_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: draft_issue_modules draft_issue_modules_created_by; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_modules
    ADD CONSTRAINT draft_issue_modules_created_by FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: draft_issue_modules draft_issue_modules_draft_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_modules
    ADD CONSTRAINT draft_issue_modules_draft_issue_id FOREIGN KEY (draft_issue_id) REFERENCES public.draft_issues(id);


--
-- Name: draft_issue_modules draft_issue_modules_module_id_4d3f477a; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_modules
    ADD CONSTRAINT draft_issue_modules_module_id_4d3f477a FOREIGN KEY (module_id) REFERENCES public.modules(id);


--
-- Name: draft_issue_modules draft_issue_modules_updated_by; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_modules
    ADD CONSTRAINT draft_issue_modules_updated_by FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: draft_issue_modules draft_issue_modules_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issue_modules
    ADD CONSTRAINT draft_issue_modules_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: draft_issues draft_issues_created_by_id_aedba72a_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issues
    ADD CONSTRAINT draft_issues_created_by_id_aedba72a_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: draft_issues draft_issues_estimate_point_id_9e333189_fk_estimate_points_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issues
    ADD CONSTRAINT draft_issues_estimate_point_id_9e333189_fk_estimate_points_id FOREIGN KEY (estimate_point_id) REFERENCES public.estimate_points(id);


--
-- Name: draft_issues draft_issues_parent_id_eee6ec32_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issues
    ADD CONSTRAINT draft_issues_parent_id_eee6ec32_fk_issues_id FOREIGN KEY (parent_id) REFERENCES public.issues(id);


--
-- Name: draft_issues draft_issues_project_id_784a560c_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issues
    ADD CONSTRAINT draft_issues_project_id_784a560c_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: draft_issues draft_issues_state_id_94f28f5a_fk_states_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issues
    ADD CONSTRAINT draft_issues_state_id_94f28f5a_fk_states_id FOREIGN KEY (state_id) REFERENCES public.states(id);


--
-- Name: draft_issues draft_issues_type_id_7a62fe34_fk_issue_types_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issues
    ADD CONSTRAINT draft_issues_type_id_7a62fe34_fk_issue_types_id FOREIGN KEY (type_id) REFERENCES public.issue_types(id);


--
-- Name: draft_issues draft_issues_updated_by_id_1ca3cd4e_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issues
    ADD CONSTRAINT draft_issues_updated_by_id_1ca3cd4e_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: draft_issues draft_issues_workspace_id_9d8512c8_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.draft_issues
    ADD CONSTRAINT draft_issues_workspace_id_9d8512c8_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: email_notification_logs email_notification_logs_created_by_id_6faff587_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.email_notification_logs
    ADD CONSTRAINT email_notification_logs_created_by_id_6faff587_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: email_notification_logs email_notification_logs_receiver_id_7c7d2e13_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.email_notification_logs
    ADD CONSTRAINT email_notification_logs_receiver_id_7c7d2e13_fk_users_id FOREIGN KEY (receiver_id) REFERENCES public.users(id);


--
-- Name: email_notification_logs email_notification_logs_triggered_by_id_b551e727_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.email_notification_logs
    ADD CONSTRAINT email_notification_logs_triggered_by_id_b551e727_fk_users_id FOREIGN KEY (triggered_by_id) REFERENCES public.users(id);


--
-- Name: email_notification_logs email_notification_logs_updated_by_id_5d99c798_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.email_notification_logs
    ADD CONSTRAINT email_notification_logs_updated_by_id_5d99c798_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: estimate_points estimate_points_created_by_id_d1b04bd9_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimate_points
    ADD CONSTRAINT estimate_points_created_by_id_d1b04bd9_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: estimate_points estimate_points_estimate_id_4b4cb706_fk_estimates_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimate_points
    ADD CONSTRAINT estimate_points_estimate_id_4b4cb706_fk_estimates_id FOREIGN KEY (estimate_id) REFERENCES public.estimates(id);


--
-- Name: estimate_points estimate_points_project_id_ba9bcb2c_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimate_points
    ADD CONSTRAINT estimate_points_project_id_ba9bcb2c_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: estimate_points estimate_points_updated_by_id_a1da94e1_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimate_points
    ADD CONSTRAINT estimate_points_updated_by_id_a1da94e1_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: estimate_points estimate_points_workspace_id_96fc4f92_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimate_points
    ADD CONSTRAINT estimate_points_workspace_id_96fc4f92_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: estimates estimates_created_by_id_7e401493_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimates
    ADD CONSTRAINT estimates_created_by_id_7e401493_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: estimates estimates_project_id_7f195a41_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimates
    ADD CONSTRAINT estimates_project_id_7f195a41_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: estimates estimates_updated_by_id_b3fcfb1d_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimates
    ADD CONSTRAINT estimates_updated_by_id_b3fcfb1d_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: estimates estimates_workspace_id_718811eb_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.estimates
    ADD CONSTRAINT estimates_workspace_id_718811eb_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: exporters exporters_created_by_id_44e1d9b3_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.exporters
    ADD CONSTRAINT exporters_created_by_id_44e1d9b3_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: exporters exporters_initiated_by_id_d51f7552_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.exporters
    ADD CONSTRAINT exporters_initiated_by_id_d51f7552_fk_users_id FOREIGN KEY (initiated_by_id) REFERENCES public.users(id);


--
-- Name: exporters exporters_updated_by_id_d2572861_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.exporters
    ADD CONSTRAINT exporters_updated_by_id_d2572861_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: exporters exporters_workspace_id_11a04317_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.exporters
    ADD CONSTRAINT exporters_workspace_id_11a04317_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: file_assets file_asset_created_by_id_966942a0_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.file_assets
    ADD CONSTRAINT file_asset_created_by_id_966942a0_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: file_assets file_asset_updated_by_id_d6aaf4f0_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.file_assets
    ADD CONSTRAINT file_asset_updated_by_id_d6aaf4f0_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: file_assets file_assets_user_id_ce1818dc_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.file_assets
    ADD CONSTRAINT file_assets_user_id_ce1818dc_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: file_assets file_assets_workspace_id_fa50b9c5_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.file_assets
    ADD CONSTRAINT file_assets_workspace_id_fa50b9c5_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: github_comment_syncs github_comment_syncs_comment_id_6feec6d1_fk_issue_comments_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_comment_syncs
    ADD CONSTRAINT github_comment_syncs_comment_id_6feec6d1_fk_issue_comments_id FOREIGN KEY (comment_id) REFERENCES public.issue_comments(id);


--
-- Name: github_comment_syncs github_comment_syncs_created_by_id_b1ef2517_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_comment_syncs
    ADD CONSTRAINT github_comment_syncs_created_by_id_b1ef2517_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: github_comment_syncs github_comment_syncs_issue_sync_id_5e738eb5_fk_github_is; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_comment_syncs
    ADD CONSTRAINT github_comment_syncs_issue_sync_id_5e738eb5_fk_github_is FOREIGN KEY (issue_sync_id) REFERENCES public.github_issue_syncs(id);


--
-- Name: github_comment_syncs github_comment_syncs_project_id_6d199ace_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_comment_syncs
    ADD CONSTRAINT github_comment_syncs_project_id_6d199ace_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: github_comment_syncs github_comment_syncs_updated_by_id_bb05c066_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_comment_syncs
    ADD CONSTRAINT github_comment_syncs_updated_by_id_bb05c066_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: github_comment_syncs github_comment_syncs_workspace_id_b54528c8_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_comment_syncs
    ADD CONSTRAINT github_comment_syncs_workspace_id_b54528c8_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: github_issue_syncs github_issue_syncs_created_by_id_d02b7c56_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_issue_syncs
    ADD CONSTRAINT github_issue_syncs_created_by_id_d02b7c56_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: github_issue_syncs github_issue_syncs_issue_id_450cb083_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_issue_syncs
    ADD CONSTRAINT github_issue_syncs_issue_id_450cb083_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: github_issue_syncs github_issue_syncs_project_id_4609ad0c_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_issue_syncs
    ADD CONSTRAINT github_issue_syncs_project_id_4609ad0c_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: github_issue_syncs github_issue_syncs_repository_sync_id_ba0d4de4_fk_github_re; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_issue_syncs
    ADD CONSTRAINT github_issue_syncs_repository_sync_id_ba0d4de4_fk_github_re FOREIGN KEY (repository_sync_id) REFERENCES public.github_repository_syncs(id);


--
-- Name: github_issue_syncs github_issue_syncs_updated_by_id_e9cd6f86_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_issue_syncs
    ADD CONSTRAINT github_issue_syncs_updated_by_id_e9cd6f86_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: github_issue_syncs github_issue_syncs_workspace_id_eae020ad_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_issue_syncs
    ADD CONSTRAINT github_issue_syncs_workspace_id_eae020ad_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: github_repositories github_repositories_created_by_id_104fa685_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repositories
    ADD CONSTRAINT github_repositories_created_by_id_104fa685_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: github_repositories github_repositories_project_id_65c546bb_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repositories
    ADD CONSTRAINT github_repositories_project_id_65c546bb_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: github_repositories github_repositories_updated_by_id_8aa4d772_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repositories
    ADD CONSTRAINT github_repositories_updated_by_id_8aa4d772_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: github_repositories github_repositories_workspace_id_c4de7326_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repositories
    ADD CONSTRAINT github_repositories_workspace_id_c4de7326_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: github_repository_syncs github_repository_sy_repository_id_ead52404_fk_github_re; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repository_syncs
    ADD CONSTRAINT github_repository_sy_repository_id_ead52404_fk_github_re FOREIGN KEY (repository_id) REFERENCES public.github_repositories(id);


--
-- Name: github_repository_syncs github_repository_sy_workspace_integratio_62858398_fk_workspace; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repository_syncs
    ADD CONSTRAINT github_repository_sy_workspace_integratio_62858398_fk_workspace FOREIGN KEY (workspace_integration_id) REFERENCES public.workspace_integrations(id);


--
-- Name: github_repository_syncs github_repository_syncs_actor_id_1fa689fe_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repository_syncs
    ADD CONSTRAINT github_repository_syncs_actor_id_1fa689fe_fk_users_id FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: github_repository_syncs github_repository_syncs_created_by_id_0df94495_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repository_syncs
    ADD CONSTRAINT github_repository_syncs_created_by_id_0df94495_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: github_repository_syncs github_repository_syncs_label_id_eb1e9bd7_fk_labels_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repository_syncs
    ADD CONSTRAINT github_repository_syncs_label_id_eb1e9bd7_fk_labels_id FOREIGN KEY (label_id) REFERENCES public.labels(id);


--
-- Name: github_repository_syncs github_repository_syncs_project_id_e7e8291e_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repository_syncs
    ADD CONSTRAINT github_repository_syncs_project_id_e7e8291e_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: github_repository_syncs github_repository_syncs_updated_by_id_07e9d065_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repository_syncs
    ADD CONSTRAINT github_repository_syncs_updated_by_id_07e9d065_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: github_repository_syncs github_repository_syncs_workspace_id_4a22a8b8_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.github_repository_syncs
    ADD CONSTRAINT github_repository_syncs_workspace_id_4a22a8b8_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: gitlab_comment_syncs gitlab_comment_syncs_comment_id_ce3343de_fk_issue_comments_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_comment_syncs
    ADD CONSTRAINT gitlab_comment_syncs_comment_id_ce3343de_fk_issue_comments_id FOREIGN KEY (comment_id) REFERENCES public.issue_comments(id);


--
-- Name: gitlab_comment_syncs gitlab_comment_syncs_created_by_id_b71e6a78_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_comment_syncs
    ADD CONSTRAINT gitlab_comment_syncs_created_by_id_b71e6a78_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: gitlab_comment_syncs gitlab_comment_syncs_issue_sync_id_933f43a8_fk_gitlab_is; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_comment_syncs
    ADD CONSTRAINT gitlab_comment_syncs_issue_sync_id_933f43a8_fk_gitlab_is FOREIGN KEY (issue_sync_id) REFERENCES public.gitlab_issue_syncs(id);


--
-- Name: gitlab_comment_syncs gitlab_comment_syncs_project_id_aea53711_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_comment_syncs
    ADD CONSTRAINT gitlab_comment_syncs_project_id_aea53711_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: gitlab_comment_syncs gitlab_comment_syncs_updated_by_id_b10af1ed_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_comment_syncs
    ADD CONSTRAINT gitlab_comment_syncs_updated_by_id_b10af1ed_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: gitlab_comment_syncs gitlab_comment_syncs_workspace_id_01365c77_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_comment_syncs
    ADD CONSTRAINT gitlab_comment_syncs_workspace_id_01365c77_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: gitlab_issue_syncs gitlab_issue_syncs_created_by_id_537fc725_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_issue_syncs
    ADD CONSTRAINT gitlab_issue_syncs_created_by_id_537fc725_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: gitlab_issue_syncs gitlab_issue_syncs_issue_id_e97a3f41_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_issue_syncs
    ADD CONSTRAINT gitlab_issue_syncs_issue_id_e97a3f41_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: gitlab_issue_syncs gitlab_issue_syncs_project_id_aff3dc2b_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_issue_syncs
    ADD CONSTRAINT gitlab_issue_syncs_project_id_aff3dc2b_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: gitlab_issue_syncs gitlab_issue_syncs_repository_sync_id_5c48ce08_fk_gitlab_re; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_issue_syncs
    ADD CONSTRAINT gitlab_issue_syncs_repository_sync_id_5c48ce08_fk_gitlab_re FOREIGN KEY (repository_sync_id) REFERENCES public.gitlab_repository_syncs(id);


--
-- Name: gitlab_issue_syncs gitlab_issue_syncs_updated_by_id_89b38a5c_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_issue_syncs
    ADD CONSTRAINT gitlab_issue_syncs_updated_by_id_89b38a5c_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: gitlab_issue_syncs gitlab_issue_syncs_workspace_id_b2d0d9da_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_issue_syncs
    ADD CONSTRAINT gitlab_issue_syncs_workspace_id_b2d0d9da_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: gitlab_repositories gitlab_repositories_created_by_id_1ea7b6cc_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repositories
    ADD CONSTRAINT gitlab_repositories_created_by_id_1ea7b6cc_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: gitlab_repositories gitlab_repositories_project_id_9d20439a_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repositories
    ADD CONSTRAINT gitlab_repositories_project_id_9d20439a_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: gitlab_repositories gitlab_repositories_updated_by_id_5a30ec6c_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repositories
    ADD CONSTRAINT gitlab_repositories_updated_by_id_5a30ec6c_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: gitlab_repositories gitlab_repositories_workspace_id_89900ac2_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repositories
    ADD CONSTRAINT gitlab_repositories_workspace_id_89900ac2_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: gitlab_repository_syncs gitlab_repository_sy_repository_id_0c59ee9a_fk_gitlab_re; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repository_syncs
    ADD CONSTRAINT gitlab_repository_sy_repository_id_0c59ee9a_fk_gitlab_re FOREIGN KEY (repository_id) REFERENCES public.gitlab_repositories(id);


--
-- Name: gitlab_repository_syncs gitlab_repository_sy_workspace_integratio_4b878644_fk_workspace; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repository_syncs
    ADD CONSTRAINT gitlab_repository_sy_workspace_integratio_4b878644_fk_workspace FOREIGN KEY (workspace_integration_id) REFERENCES public.workspace_integrations(id);


--
-- Name: gitlab_repository_syncs gitlab_repository_syncs_actor_id_0f7b2f41_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repository_syncs
    ADD CONSTRAINT gitlab_repository_syncs_actor_id_0f7b2f41_fk_users_id FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: gitlab_repository_syncs gitlab_repository_syncs_created_by_id_51a64cc3_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repository_syncs
    ADD CONSTRAINT gitlab_repository_syncs_created_by_id_51a64cc3_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: gitlab_repository_syncs gitlab_repository_syncs_label_id_a4aa3f90_fk_labels_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repository_syncs
    ADD CONSTRAINT gitlab_repository_syncs_label_id_a4aa3f90_fk_labels_id FOREIGN KEY (label_id) REFERENCES public.labels(id);


--
-- Name: gitlab_repository_syncs gitlab_repository_syncs_project_id_9d61576c_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repository_syncs
    ADD CONSTRAINT gitlab_repository_syncs_project_id_9d61576c_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: gitlab_repository_syncs gitlab_repository_syncs_updated_by_id_06371794_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repository_syncs
    ADD CONSTRAINT gitlab_repository_syncs_updated_by_id_06371794_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: gitlab_repository_syncs gitlab_repository_syncs_workspace_id_e72ee0b4_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.gitlab_repository_syncs
    ADD CONSTRAINT gitlab_repository_syncs_workspace_id_e72ee0b4_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: importers importers_created_by_id_7dd06433_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.importers
    ADD CONSTRAINT importers_created_by_id_7dd06433_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: importers importers_initiated_by_id_3cddbd23_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.importers
    ADD CONSTRAINT importers_initiated_by_id_3cddbd23_fk_users_id FOREIGN KEY (initiated_by_id) REFERENCES public.users(id);


--
-- Name: importers importers_project_id_1f8b43ef_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.importers
    ADD CONSTRAINT importers_project_id_1f8b43ef_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: importers importers_token_id_c951e89f_fk_api_tokens_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.importers
    ADD CONSTRAINT importers_token_id_c951e89f_fk_api_tokens_id FOREIGN KEY (token_id) REFERENCES public.api_tokens(id);


--
-- Name: importers importers_updated_by_id_3915139e_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.importers
    ADD CONSTRAINT importers_updated_by_id_3915139e_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: importers importers_workspace_id_795b8985_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.importers
    ADD CONSTRAINT importers_workspace_id_795b8985_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: intake_issues inbox_issues_created_by_id_483bce13_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intake_issues
    ADD CONSTRAINT inbox_issues_created_by_id_483bce13_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: intake_issues inbox_issues_duplicate_to_id_6cb8d961_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intake_issues
    ADD CONSTRAINT inbox_issues_duplicate_to_id_6cb8d961_fk_issues_id FOREIGN KEY (duplicate_to_id) REFERENCES public.issues(id);


--
-- Name: intake_issues inbox_issues_intake_id_a04a7455_fk_intakes_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intake_issues
    ADD CONSTRAINT inbox_issues_intake_id_a04a7455_fk_intakes_id FOREIGN KEY (intake_id) REFERENCES public.intakes(id);


--
-- Name: intake_issues inbox_issues_issue_id_7d74b224_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intake_issues
    ADD CONSTRAINT inbox_issues_issue_id_7d74b224_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: intake_issues inbox_issues_project_id_5117a70b_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intake_issues
    ADD CONSTRAINT inbox_issues_project_id_5117a70b_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: intake_issues inbox_issues_updated_by_id_d1b2b70f_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intake_issues
    ADD CONSTRAINT inbox_issues_updated_by_id_d1b2b70f_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: intake_issues inbox_issues_workspace_id_4a61a7bd_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intake_issues
    ADD CONSTRAINT inbox_issues_workspace_id_4a61a7bd_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: intakes inboxes_created_by_id_9f1cf5ec_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intakes
    ADD CONSTRAINT inboxes_created_by_id_9f1cf5ec_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: intakes inboxes_project_id_a0135c66_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intakes
    ADD CONSTRAINT inboxes_project_id_a0135c66_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: intakes inboxes_updated_by_id_69b7b3ae_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intakes
    ADD CONSTRAINT inboxes_updated_by_id_69b7b3ae_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: intakes inboxes_workspace_id_d6178865_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.intakes
    ADD CONSTRAINT inboxes_workspace_id_d6178865_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: instance_admins instance_admins_created_by_id_7f4e03b4_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instance_admins
    ADD CONSTRAINT instance_admins_created_by_id_7f4e03b4_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: instance_admins instance_admins_instance_id_66d1ba73_fk_instances_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instance_admins
    ADD CONSTRAINT instance_admins_instance_id_66d1ba73_fk_instances_id FOREIGN KEY (instance_id) REFERENCES public.instances(id);


--
-- Name: instance_admins instance_admins_updated_by_id_b7800403_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instance_admins
    ADD CONSTRAINT instance_admins_updated_by_id_b7800403_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: instance_admins instance_admins_user_id_cc6e9b62_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instance_admins
    ADD CONSTRAINT instance_admins_user_id_cc6e9b62_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: instance_configurations instance_configurations_created_by_id_e683f3e5_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instance_configurations
    ADD CONSTRAINT instance_configurations_created_by_id_e683f3e5_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: instance_configurations instance_configurations_updated_by_id_f0d7542e_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instance_configurations
    ADD CONSTRAINT instance_configurations_updated_by_id_f0d7542e_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: instances instances_created_by_id_c76e92ef_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instances
    ADD CONSTRAINT instances_created_by_id_c76e92ef_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: instances instances_updated_by_id_cce8fcdf_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.instances
    ADD CONSTRAINT instances_updated_by_id_cce8fcdf_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: integrations integrations_created_by_id_0b6edd52_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.integrations
    ADD CONSTRAINT integrations_created_by_id_0b6edd52_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: integrations integrations_updated_by_id_d6d00d15_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.integrations
    ADD CONSTRAINT integrations_updated_by_id_d6d00d15_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_activities issue_activities_issue_id_180e5662_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_activities
    ADD CONSTRAINT issue_activities_issue_id_180e5662_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_activities issue_activity_actor_id_52fdd42d_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_activities
    ADD CONSTRAINT issue_activity_actor_id_52fdd42d_fk_user_id FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: issue_activities issue_activity_created_by_id_49516e3d_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_activities
    ADD CONSTRAINT issue_activity_created_by_id_49516e3d_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_activities issue_activity_issue_comment_id_701f3c3c_fk_issue_comment_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_activities
    ADD CONSTRAINT issue_activity_issue_comment_id_701f3c3c_fk_issue_comment_id FOREIGN KEY (issue_comment_id) REFERENCES public.issue_comments(id);


--
-- Name: issue_activities issue_activity_project_id_d0ac2ccf_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_activities
    ADD CONSTRAINT issue_activity_project_id_d0ac2ccf_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_activities issue_activity_updated_by_id_0075f9bd_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_activities
    ADD CONSTRAINT issue_activity_updated_by_id_0075f9bd_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_activities issue_activity_workspace_id_65acaf73_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_activities
    ADD CONSTRAINT issue_activity_workspace_id_65acaf73_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_assignees issue_assignee_assignee_id_50f5c04e_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_assignees
    ADD CONSTRAINT issue_assignee_assignee_id_50f5c04e_fk_user_id FOREIGN KEY (assignee_id) REFERENCES public.users(id);


--
-- Name: issue_assignees issue_assignee_created_by_id_f693d43b_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_assignees
    ADD CONSTRAINT issue_assignee_created_by_id_f693d43b_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_assignees issue_assignee_issue_id_72da08db_fk_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_assignees
    ADD CONSTRAINT issue_assignee_issue_id_72da08db_fk_issue_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_assignees issue_assignee_project_id_61c18bf2_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_assignees
    ADD CONSTRAINT issue_assignee_project_id_61c18bf2_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_assignees issue_assignee_updated_by_id_c54088aa_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_assignees
    ADD CONSTRAINT issue_assignee_updated_by_id_c54088aa_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_assignees issue_assignee_workspace_id_9aad55b7_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_assignees
    ADD CONSTRAINT issue_assignee_workspace_id_9aad55b7_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_attachments issue_attachments_created_by_id_87be05bb_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_attachments
    ADD CONSTRAINT issue_attachments_created_by_id_87be05bb_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_attachments issue_attachments_issue_id_0faf88bf_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_attachments
    ADD CONSTRAINT issue_attachments_issue_id_0faf88bf_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_attachments issue_attachments_project_id_a95fe706_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_attachments
    ADD CONSTRAINT issue_attachments_project_id_a95fe706_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_attachments issue_attachments_updated_by_id_47dceec1_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_attachments
    ADD CONSTRAINT issue_attachments_updated_by_id_47dceec1_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_attachments issue_attachments_workspace_id_c456a532_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_attachments
    ADD CONSTRAINT issue_attachments_workspace_id_c456a532_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_blockers issue_blocker_block_id_5d15a701_fk_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_blockers
    ADD CONSTRAINT issue_blocker_block_id_5d15a701_fk_issue_id FOREIGN KEY (block_id) REFERENCES public.issues(id);


--
-- Name: issue_blockers issue_blocker_blocked_by_id_a138af71_fk_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_blockers
    ADD CONSTRAINT issue_blocker_blocked_by_id_a138af71_fk_issue_id FOREIGN KEY (blocked_by_id) REFERENCES public.issues(id);


--
-- Name: issue_blockers issue_blocker_created_by_id_0d19f6ea_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_blockers
    ADD CONSTRAINT issue_blocker_created_by_id_0d19f6ea_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_blockers issue_blocker_project_id_380bd100_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_blockers
    ADD CONSTRAINT issue_blocker_project_id_380bd100_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_blockers issue_blocker_updated_by_id_4af87d63_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_blockers
    ADD CONSTRAINT issue_blocker_updated_by_id_4af87d63_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_blockers issue_blocker_workspace_id_419a1c71_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_blockers
    ADD CONSTRAINT issue_blocker_workspace_id_419a1c71_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_comments issue_comment_actor_id_d312315b_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_comments
    ADD CONSTRAINT issue_comment_actor_id_d312315b_fk_user_id FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: issue_comments issue_comment_created_by_id_0765f239_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_comments
    ADD CONSTRAINT issue_comment_created_by_id_0765f239_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_comments issue_comment_issue_id_d0195e35_fk_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_comments
    ADD CONSTRAINT issue_comment_issue_id_d0195e35_fk_issue_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_comments issue_comment_project_id_db37c105_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_comments
    ADD CONSTRAINT issue_comment_project_id_db37c105_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_comments issue_comment_updated_by_id_96cfb86e_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_comments
    ADD CONSTRAINT issue_comment_updated_by_id_96cfb86e_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_comments issue_comment_workspace_id_3f7969ec_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_comments
    ADD CONSTRAINT issue_comment_workspace_id_3f7969ec_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_comments issue_comments_description_id_0cb72512_fk_descriptions_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_comments
    ADD CONSTRAINT issue_comments_description_id_0cb72512_fk_descriptions_id FOREIGN KEY (description_id) REFERENCES public.descriptions(id);


--
-- Name: issue_comments issue_comments_parent_id_d8db10b1_fk_issue_comments_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_comments
    ADD CONSTRAINT issue_comments_parent_id_d8db10b1_fk_issue_comments_id FOREIGN KEY (parent_id) REFERENCES public.issue_comments(id);


--
-- Name: issues issue_created_by_id_8f0ae62b_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issues
    ADD CONSTRAINT issue_created_by_id_8f0ae62b_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_description_versions issue_description_ve_workspace_id_88e930f9_fk_workspace; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_description_versions
    ADD CONSTRAINT issue_description_ve_workspace_id_88e930f9_fk_workspace FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_description_versions issue_description_versions_created_by_id_3f7e62a1_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_description_versions
    ADD CONSTRAINT issue_description_versions_created_by_id_3f7e62a1_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_description_versions issue_description_versions_issue_id_c8baa13e_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_description_versions
    ADD CONSTRAINT issue_description_versions_issue_id_c8baa13e_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_description_versions issue_description_versions_owned_by_id_0effe4d0_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_description_versions
    ADD CONSTRAINT issue_description_versions_owned_by_id_0effe4d0_fk_users_id FOREIGN KEY (owned_by_id) REFERENCES public.users(id);


--
-- Name: issue_description_versions issue_description_versions_project_id_536b23ef_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_description_versions
    ADD CONSTRAINT issue_description_versions_project_id_536b23ef_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_description_versions issue_description_versions_updated_by_id_6530365d_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_description_versions
    ADD CONSTRAINT issue_description_versions_updated_by_id_6530365d_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_labels issue_label_created_by_id_94075315_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_labels
    ADD CONSTRAINT issue_label_created_by_id_94075315_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_labels issue_label_issue_id_0f252e52_fk_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_labels
    ADD CONSTRAINT issue_label_issue_id_0f252e52_fk_issue_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_labels issue_label_label_id_5f22777f_fk_label_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_labels
    ADD CONSTRAINT issue_label_label_id_5f22777f_fk_label_id FOREIGN KEY (label_id) REFERENCES public.labels(id);


--
-- Name: issue_labels issue_label_project_id_eaa2ba39_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_labels
    ADD CONSTRAINT issue_label_project_id_eaa2ba39_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_labels issue_label_updated_by_id_a97a6733_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_labels
    ADD CONSTRAINT issue_label_updated_by_id_a97a6733_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_labels issue_label_workspace_id_b5b1faac_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_labels
    ADD CONSTRAINT issue_label_workspace_id_b5b1faac_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_links issue_links_created_by_id_5e4aa092_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_links
    ADD CONSTRAINT issue_links_created_by_id_5e4aa092_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_links issue_links_issue_id_7032881f_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_links
    ADD CONSTRAINT issue_links_issue_id_7032881f_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_links issue_links_project_id_63d6e9ce_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_links
    ADD CONSTRAINT issue_links_project_id_63d6e9ce_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_links issue_links_updated_by_id_a771cce4_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_links
    ADD CONSTRAINT issue_links_updated_by_id_a771cce4_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_links issue_links_workspace_id_ff9038e7_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_links
    ADD CONSTRAINT issue_links_workspace_id_ff9038e7_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_mentions issue_mentions_created_by_id_eb44759e_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_mentions
    ADD CONSTRAINT issue_mentions_created_by_id_eb44759e_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_mentions issue_mentions_issue_id_d8821107_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_mentions
    ADD CONSTRAINT issue_mentions_issue_id_d8821107_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_mentions issue_mentions_mention_id_cf1b9346_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_mentions
    ADD CONSTRAINT issue_mentions_mention_id_cf1b9346_fk_users_id FOREIGN KEY (mention_id) REFERENCES public.users(id);


--
-- Name: issue_mentions issue_mentions_project_id_d0cccdf5_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_mentions
    ADD CONSTRAINT issue_mentions_project_id_d0cccdf5_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_mentions issue_mentions_updated_by_id_c62106d3_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_mentions
    ADD CONSTRAINT issue_mentions_updated_by_id_c62106d3_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_mentions issue_mentions_workspace_id_4ca59d05_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_mentions
    ADD CONSTRAINT issue_mentions_workspace_id_4ca59d05_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issues issue_parent_id_ce8d76ba_fk_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issues
    ADD CONSTRAINT issue_parent_id_ce8d76ba_fk_issue_id FOREIGN KEY (parent_id) REFERENCES public.issues(id);


--
-- Name: issues issue_project_id_fea0fc80_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issues
    ADD CONSTRAINT issue_project_id_fea0fc80_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: project_user_properties issue_property_created_by_id_8e92131c_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_user_properties
    ADD CONSTRAINT issue_property_created_by_id_8e92131c_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: project_user_properties issue_property_project_id_30e7de7b_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_user_properties
    ADD CONSTRAINT issue_property_project_id_30e7de7b_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: project_user_properties issue_property_updated_by_id_ff158d4d_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_user_properties
    ADD CONSTRAINT issue_property_updated_by_id_ff158d4d_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: project_user_properties issue_property_user_id_0b1d1c8f_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_user_properties
    ADD CONSTRAINT issue_property_user_id_0b1d1c8f_fk_user_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: project_user_properties issue_property_workspace_id_17860d65_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_user_properties
    ADD CONSTRAINT issue_property_workspace_id_17860d65_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_reactions issue_reactions_actor_id_5f5b8303_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_reactions
    ADD CONSTRAINT issue_reactions_actor_id_5f5b8303_fk_users_id FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: issue_reactions issue_reactions_created_by_id_3953b7de_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_reactions
    ADD CONSTRAINT issue_reactions_created_by_id_3953b7de_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_reactions issue_reactions_issue_id_2c324bae_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_reactions
    ADD CONSTRAINT issue_reactions_issue_id_2c324bae_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_reactions issue_reactions_project_id_8708ecaf_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_reactions
    ADD CONSTRAINT issue_reactions_project_id_8708ecaf_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_reactions issue_reactions_updated_by_id_4069af90_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_reactions
    ADD CONSTRAINT issue_reactions_updated_by_id_4069af90_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_reactions issue_reactions_workspace_id_bd8d7550_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_reactions
    ADD CONSTRAINT issue_reactions_workspace_id_bd8d7550_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_relations issue_relations_created_by_id_854d07e7_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_relations
    ADD CONSTRAINT issue_relations_created_by_id_854d07e7_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_relations issue_relations_issue_id_e1db6f72_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_relations
    ADD CONSTRAINT issue_relations_issue_id_e1db6f72_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_relations issue_relations_project_id_15350161_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_relations
    ADD CONSTRAINT issue_relations_project_id_15350161_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_relations issue_relations_related_issue_id_e1ea44a7_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_relations
    ADD CONSTRAINT issue_relations_related_issue_id_e1ea44a7_fk_issues_id FOREIGN KEY (related_issue_id) REFERENCES public.issues(id);


--
-- Name: issue_relations issue_relations_updated_by_id_3dfa850f_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_relations
    ADD CONSTRAINT issue_relations_updated_by_id_3dfa850f_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_relations issue_relations_workspace_id_00b50e90_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_relations
    ADD CONSTRAINT issue_relations_workspace_id_00b50e90_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_sequences issue_sequence_created_by_id_59270506_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_sequences
    ADD CONSTRAINT issue_sequence_created_by_id_59270506_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_sequences issue_sequence_issue_id_16e9f00f_fk_issue_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_sequences
    ADD CONSTRAINT issue_sequence_issue_id_16e9f00f_fk_issue_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_sequences issue_sequence_project_id_ce882e85_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_sequences
    ADD CONSTRAINT issue_sequence_project_id_ce882e85_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_sequences issue_sequence_updated_by_id_310c8dd3_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_sequences
    ADD CONSTRAINT issue_sequence_updated_by_id_310c8dd3_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_sequences issue_sequence_workspace_id_0d3f0fd4_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_sequences
    ADD CONSTRAINT issue_sequence_workspace_id_0d3f0fd4_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issues issue_state_id_1a65560d_fk_state_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issues
    ADD CONSTRAINT issue_state_id_1a65560d_fk_state_id FOREIGN KEY (state_id) REFERENCES public.states(id);


--
-- Name: issue_subscribers issue_subscribers_created_by_id_b6ea0157_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_subscribers
    ADD CONSTRAINT issue_subscribers_created_by_id_b6ea0157_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_subscribers issue_subscribers_issue_id_85cf2093_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_subscribers
    ADD CONSTRAINT issue_subscribers_issue_id_85cf2093_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_subscribers issue_subscribers_project_id_cf48d75f_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_subscribers
    ADD CONSTRAINT issue_subscribers_project_id_cf48d75f_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_subscribers issue_subscribers_subscriber_id_2d89c988_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_subscribers
    ADD CONSTRAINT issue_subscribers_subscriber_id_2d89c988_fk_users_id FOREIGN KEY (subscriber_id) REFERENCES public.users(id);


--
-- Name: issue_subscribers issue_subscribers_updated_by_id_1bfc2f55_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_subscribers
    ADD CONSTRAINT issue_subscribers_updated_by_id_1bfc2f55_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_subscribers issue_subscribers_workspace_id_96afa91f_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_subscribers
    ADD CONSTRAINT issue_subscribers_workspace_id_96afa91f_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_types issue_types_created_by_id_48764f53_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_types
    ADD CONSTRAINT issue_types_created_by_id_48764f53_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_types issue_types_updated_by_id_4919203b_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_types
    ADD CONSTRAINT issue_types_updated_by_id_4919203b_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_types issue_types_workspace_id_591c6f3b_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_types
    ADD CONSTRAINT issue_types_workspace_id_591c6f3b_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issues issue_updated_by_id_f1261863_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issues
    ADD CONSTRAINT issue_updated_by_id_f1261863_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_versions issue_versions_activity_id_b1872ffc_fk_issue_activities_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_versions
    ADD CONSTRAINT issue_versions_activity_id_b1872ffc_fk_issue_activities_id FOREIGN KEY (activity_id) REFERENCES public.issue_activities(id);


--
-- Name: issue_versions issue_versions_created_by_id_a782830a_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_versions
    ADD CONSTRAINT issue_versions_created_by_id_a782830a_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_versions issue_versions_issue_id_25cf001c_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_versions
    ADD CONSTRAINT issue_versions_issue_id_25cf001c_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_versions issue_versions_owned_by_id_7586378d_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_versions
    ADD CONSTRAINT issue_versions_owned_by_id_7586378d_fk_users_id FOREIGN KEY (owned_by_id) REFERENCES public.users(id);


--
-- Name: issue_versions issue_versions_project_id_a069ad03_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_versions
    ADD CONSTRAINT issue_versions_project_id_a069ad03_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_versions issue_versions_updated_by_id_dcae6dd2_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_versions
    ADD CONSTRAINT issue_versions_updated_by_id_dcae6dd2_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_versions issue_versions_workspace_id_b8c48b7c_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_versions
    ADD CONSTRAINT issue_versions_workspace_id_b8c48b7c_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_views issue_views_created_by_id_0d2e456b_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_views
    ADD CONSTRAINT issue_views_created_by_id_0d2e456b_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_views issue_views_owned_by_id_5e261e5d_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_views
    ADD CONSTRAINT issue_views_owned_by_id_5e261e5d_fk_users_id FOREIGN KEY (owned_by_id) REFERENCES public.users(id);


--
-- Name: issue_views issue_views_project_id_55ee009f_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_views
    ADD CONSTRAINT issue_views_project_id_55ee009f_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_views issue_views_updated_by_id_28cd9870_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_views
    ADD CONSTRAINT issue_views_updated_by_id_28cd9870_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_views issue_views_workspace_id_8785e03d_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_views
    ADD CONSTRAINT issue_views_workspace_id_8785e03d_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issue_votes issue_votes_actor_id_525cab61_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_votes
    ADD CONSTRAINT issue_votes_actor_id_525cab61_fk_users_id FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: issue_votes issue_votes_created_by_id_86adcf5c_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_votes
    ADD CONSTRAINT issue_votes_created_by_id_86adcf5c_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: issue_votes issue_votes_issue_id_07a61ecb_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_votes
    ADD CONSTRAINT issue_votes_issue_id_07a61ecb_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: issue_votes issue_votes_project_id_b649f55b_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_votes
    ADD CONSTRAINT issue_votes_project_id_b649f55b_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: issue_votes issue_votes_updated_by_id_9e2a6cdc_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_votes
    ADD CONSTRAINT issue_votes_updated_by_id_9e2a6cdc_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: issue_votes issue_votes_workspace_id_a3e91a6b_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issue_votes
    ADD CONSTRAINT issue_votes_workspace_id_a3e91a6b_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issues issue_workspace_id_c84878c1_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issues
    ADD CONSTRAINT issue_workspace_id_c84878c1_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: issues issues_estimate_point_id_a6822abe_fk_estimate_points_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issues
    ADD CONSTRAINT issues_estimate_point_id_a6822abe_fk_estimate_points_id FOREIGN KEY (estimate_point_id) REFERENCES public.estimate_points(id);


--
-- Name: issues issues_type_id_a4710b19_fk_issue_types_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.issues
    ADD CONSTRAINT issues_type_id_a4710b19_fk_issue_types_id FOREIGN KEY (type_id) REFERENCES public.issue_types(id);


--
-- Name: labels label_created_by_id_aa6ffcfa_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.labels
    ADD CONSTRAINT label_created_by_id_aa6ffcfa_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: labels label_parent_id_7a853296_fk_label_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.labels
    ADD CONSTRAINT label_parent_id_7a853296_fk_label_id FOREIGN KEY (parent_id) REFERENCES public.labels(id);


--
-- Name: labels label_updated_by_id_894a5464_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.labels
    ADD CONSTRAINT label_updated_by_id_894a5464_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: labels label_workspace_id_c4c9ae5a_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.labels
    ADD CONSTRAINT label_workspace_id_c4c9ae5a_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: labels labels_project_id_cf57a802_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.labels
    ADD CONSTRAINT labels_project_id_cf57a802_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: modules module_created_by_id_ff7a5866_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.modules
    ADD CONSTRAINT module_created_by_id_ff7a5866_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: module_issues module_issues_created_by_id_de0b995a_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_issues
    ADD CONSTRAINT module_issues_created_by_id_de0b995a_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: module_issues module_issues_issue_id_7caa908b_fk_issues_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_issues
    ADD CONSTRAINT module_issues_issue_id_7caa908b_fk_issues_id FOREIGN KEY (issue_id) REFERENCES public.issues(id);


--
-- Name: module_issues module_issues_module_id_74e0ed5a_fk_module_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_issues
    ADD CONSTRAINT module_issues_module_id_74e0ed5a_fk_module_id FOREIGN KEY (module_id) REFERENCES public.modules(id);


--
-- Name: module_issues module_issues_project_id_59836d1e_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_issues
    ADD CONSTRAINT module_issues_project_id_59836d1e_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: module_issues module_issues_updated_by_id_46dbf724_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_issues
    ADD CONSTRAINT module_issues_updated_by_id_46dbf724_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: module_issues module_issues_workspace_id_6bf85201_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_issues
    ADD CONSTRAINT module_issues_workspace_id_6bf85201_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: modules module_lead_id_04966630_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.modules
    ADD CONSTRAINT module_lead_id_04966630_fk_user_id FOREIGN KEY (lead_id) REFERENCES public.users(id);


--
-- Name: module_links module_links_created_by_id_eaf6492f_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_links
    ADD CONSTRAINT module_links_created_by_id_eaf6492f_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: module_links module_links_module_id_0fda3f8a_fk_modules_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_links
    ADD CONSTRAINT module_links_module_id_0fda3f8a_fk_modules_id FOREIGN KEY (module_id) REFERENCES public.modules(id);


--
-- Name: module_links module_links_project_id_f720bb79_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_links
    ADD CONSTRAINT module_links_project_id_f720bb79_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: module_links module_links_updated_by_id_4da419e7_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_links
    ADD CONSTRAINT module_links_updated_by_id_4da419e7_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: module_links module_links_workspace_id_0521c11c_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_links
    ADD CONSTRAINT module_links_workspace_id_0521c11c_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: module_members module_member_created_by_id_2ed84a65_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_members
    ADD CONSTRAINT module_member_created_by_id_2ed84a65_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: module_members module_member_member_id_928f473e_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_members
    ADD CONSTRAINT module_member_member_id_928f473e_fk_user_id FOREIGN KEY (member_id) REFERENCES public.users(id);


--
-- Name: module_members module_member_module_id_f00be7ef_fk_module_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_members
    ADD CONSTRAINT module_member_module_id_f00be7ef_fk_module_id FOREIGN KEY (module_id) REFERENCES public.modules(id);


--
-- Name: module_members module_member_project_id_ec8d2376_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_members
    ADD CONSTRAINT module_member_project_id_ec8d2376_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: module_members module_member_updated_by_id_a9046438_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_members
    ADD CONSTRAINT module_member_updated_by_id_a9046438_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: module_members module_member_workspace_id_f2f23c73_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_members
    ADD CONSTRAINT module_member_workspace_id_f2f23c73_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: modules module_project_id_da84b04f_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.modules
    ADD CONSTRAINT module_project_id_da84b04f_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: modules module_updated_by_id_72ab6d5c_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.modules
    ADD CONSTRAINT module_updated_by_id_72ab6d5c_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: module_user_properties module_user_properties_created_by_id_bdd98440_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_user_properties
    ADD CONSTRAINT module_user_properties_created_by_id_bdd98440_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: module_user_properties module_user_properties_module_id_e95b158a_fk_modules_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_user_properties
    ADD CONSTRAINT module_user_properties_module_id_e95b158a_fk_modules_id FOREIGN KEY (module_id) REFERENCES public.modules(id);


--
-- Name: module_user_properties module_user_properties_project_id_3c5a4972_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_user_properties
    ADD CONSTRAINT module_user_properties_project_id_3c5a4972_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: module_user_properties module_user_properties_updated_by_id_b7dafc77_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_user_properties
    ADD CONSTRAINT module_user_properties_updated_by_id_b7dafc77_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: module_user_properties module_user_properties_user_id_e83a1c2c_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_user_properties
    ADD CONSTRAINT module_user_properties_user_id_e83a1c2c_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: module_user_properties module_user_properties_workspace_id_ddaf807c_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.module_user_properties
    ADD CONSTRAINT module_user_properties_workspace_id_ddaf807c_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: modules module_workspace_id_0a826fef_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.modules
    ADD CONSTRAINT module_workspace_id_0a826fef_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: notifications notifications_created_by_id_b9c3f81b_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_created_by_id_b9c3f81b_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: notifications notifications_project_id_e4d4f192_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_project_id_e4d4f192_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: notifications notifications_receiver_id_b708b2b0_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_receiver_id_b708b2b0_fk_users_id FOREIGN KEY (receiver_id) REFERENCES public.users(id);


--
-- Name: notifications notifications_triggered_by_id_31cdec21_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_triggered_by_id_31cdec21_fk_users_id FOREIGN KEY (triggered_by_id) REFERENCES public.users(id);


--
-- Name: notifications notifications_updated_by_id_8a651e96_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_updated_by_id_8a651e96_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: notifications notifications_workspace_id_b2f09ef7_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT notifications_workspace_id_b2f09ef7_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: page_labels page_labels_created_by_id_fbd942c0_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_labels
    ADD CONSTRAINT page_labels_created_by_id_fbd942c0_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: page_labels page_labels_label_id_05958e53_fk_labels_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_labels
    ADD CONSTRAINT page_labels_label_id_05958e53_fk_labels_id FOREIGN KEY (label_id) REFERENCES public.labels(id);


--
-- Name: page_labels page_labels_page_id_0e6cdb3d_fk_pages_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_labels
    ADD CONSTRAINT page_labels_page_id_0e6cdb3d_fk_pages_id FOREIGN KEY (page_id) REFERENCES public.pages(id);


--
-- Name: page_labels page_labels_updated_by_id_d9fddbff_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_labels
    ADD CONSTRAINT page_labels_updated_by_id_d9fddbff_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: page_labels page_labels_workspace_id_078bb01c_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_labels
    ADD CONSTRAINT page_labels_workspace_id_078bb01c_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: page_logs page_logs_created_by_id_4a295aec_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_logs
    ADD CONSTRAINT page_logs_created_by_id_4a295aec_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: page_logs page_logs_page_id_0e0d747d_fk_pages_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_logs
    ADD CONSTRAINT page_logs_page_id_0e0d747d_fk_pages_id FOREIGN KEY (page_id) REFERENCES public.pages(id);


--
-- Name: page_logs page_logs_updated_by_id_1995190b_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_logs
    ADD CONSTRAINT page_logs_updated_by_id_1995190b_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: page_logs page_logs_workspace_id_be7bde64_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_logs
    ADD CONSTRAINT page_logs_workspace_id_be7bde64_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: page_versions page_versions_created_by_id_d660b13b_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_versions
    ADD CONSTRAINT page_versions_created_by_id_d660b13b_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: page_versions page_versions_owned_by_id_6d9143db_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_versions
    ADD CONSTRAINT page_versions_owned_by_id_6d9143db_fk_users_id FOREIGN KEY (owned_by_id) REFERENCES public.users(id);


--
-- Name: page_versions page_versions_page_id_c46471da_fk_pages_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_versions
    ADD CONSTRAINT page_versions_page_id_c46471da_fk_pages_id FOREIGN KEY (page_id) REFERENCES public.pages(id);


--
-- Name: page_versions page_versions_updated_by_id_72d5e579_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_versions
    ADD CONSTRAINT page_versions_updated_by_id_72d5e579_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: page_versions page_versions_workspace_id_8330a200_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.page_versions
    ADD CONSTRAINT page_versions_workspace_id_8330a200_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: pages pages_created_by_id_d109a675_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_created_by_id_d109a675_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: pages pages_owned_by_id_bf50485f_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_owned_by_id_bf50485f_fk_users_id FOREIGN KEY (owned_by_id) REFERENCES public.users(id);


--
-- Name: pages pages_parent_id_8b823409_fk_pages_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_parent_id_8b823409_fk_pages_id FOREIGN KEY (parent_id) REFERENCES public.pages(id);


--
-- Name: pages pages_updated_by_id_6c42de3e_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_updated_by_id_6c42de3e_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: pages pages_workspace_id_c6c51010_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.pages
    ADD CONSTRAINT pages_workspace_id_c6c51010_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: profiles profiles_user_id_36580373_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.profiles
    ADD CONSTRAINT profiles_user_id_36580373_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: projects project_created_by_id_6cc13408_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT project_created_by_id_6cc13408_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: projects project_default_assignee_id_6ba45f90_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT project_default_assignee_id_6ba45f90_fk_user_id FOREIGN KEY (default_assignee_id) REFERENCES public.users(id);


--
-- Name: project_deploy_boards project_deploy_boards_created_by_id_2ea72f98_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_deploy_boards
    ADD CONSTRAINT project_deploy_boards_created_by_id_2ea72f98_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: project_deploy_boards project_deploy_boards_intake_id_36aa612d_fk_intakes_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_deploy_boards
    ADD CONSTRAINT project_deploy_boards_intake_id_36aa612d_fk_intakes_id FOREIGN KEY (intake_id) REFERENCES public.intakes(id);


--
-- Name: project_deploy_boards project_deploy_boards_project_id_49d887b2_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_deploy_boards
    ADD CONSTRAINT project_deploy_boards_project_id_49d887b2_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: project_deploy_boards project_deploy_boards_updated_by_id_290eb99e_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_deploy_boards
    ADD CONSTRAINT project_deploy_boards_updated_by_id_290eb99e_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: project_deploy_boards project_deploy_boards_workspace_id_cd92f164_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_deploy_boards
    ADD CONSTRAINT project_deploy_boards_workspace_id_cd92f164_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: project_identifiers project_identifier_created_by_id_2b6f273a_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_identifiers
    ADD CONSTRAINT project_identifier_created_by_id_2b6f273a_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: project_identifiers project_identifier_project_id_13de58a9_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_identifiers
    ADD CONSTRAINT project_identifier_project_id_13de58a9_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: project_identifiers project_identifier_updated_by_id_1a00e2a0_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_identifiers
    ADD CONSTRAINT project_identifier_updated_by_id_1a00e2a0_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: project_identifiers project_identifier_workspace_id_6024b517_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_identifiers
    ADD CONSTRAINT project_identifier_workspace_id_6024b517_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: project_issue_types project_issue_types_created_by_id_049cecfd_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_issue_types
    ADD CONSTRAINT project_issue_types_created_by_id_049cecfd_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: project_issue_types project_issue_types_issue_type_id_9494de9f_fk_issue_types_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_issue_types
    ADD CONSTRAINT project_issue_types_issue_type_id_9494de9f_fk_issue_types_id FOREIGN KEY (issue_type_id) REFERENCES public.issue_types(id);


--
-- Name: project_issue_types project_issue_types_project_id_ef6e52e4_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_issue_types
    ADD CONSTRAINT project_issue_types_project_id_ef6e52e4_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: project_issue_types project_issue_types_updated_by_id_b5998397_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_issue_types
    ADD CONSTRAINT project_issue_types_updated_by_id_b5998397_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: project_issue_types project_issue_types_workspace_id_ace3c5b5_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_issue_types
    ADD CONSTRAINT project_issue_types_workspace_id_ace3c5b5_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: project_members project_member_created_by_id_8b363306_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_member_created_by_id_8b363306_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: project_member_invites project_member_invite_created_by_id_a87df45c_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_member_invites
    ADD CONSTRAINT project_member_invite_created_by_id_a87df45c_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: project_member_invites project_member_invite_project_id_8fb7750e_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_member_invites
    ADD CONSTRAINT project_member_invite_project_id_8fb7750e_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: project_member_invites project_member_invite_updated_by_id_5aa55c96_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_member_invites
    ADD CONSTRAINT project_member_invite_updated_by_id_5aa55c96_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: project_member_invites project_member_invite_workspace_id_64e2dc4c_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_member_invites
    ADD CONSTRAINT project_member_invite_workspace_id_64e2dc4c_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: project_members project_member_member_id_9d6b126b_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_member_member_id_9d6b126b_fk_user_id FOREIGN KEY (member_id) REFERENCES public.users(id);


--
-- Name: project_members project_member_project_id_11ea1a9e_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_member_project_id_11ea1a9e_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: project_members project_member_updated_by_id_cf6aaac4_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_member_updated_by_id_cf6aaac4_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: project_members project_member_workspace_id_88bb9a97_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_members
    ADD CONSTRAINT project_member_workspace_id_88bb9a97_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: project_pages project_pages_created_by_id_b9d02062_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_pages
    ADD CONSTRAINT project_pages_created_by_id_b9d02062_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: project_pages project_pages_page_id_a0f54439_fk_pages_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_pages
    ADD CONSTRAINT project_pages_page_id_a0f54439_fk_pages_id FOREIGN KEY (page_id) REFERENCES public.pages(id);


--
-- Name: project_pages project_pages_project_id_376ba35a_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_pages
    ADD CONSTRAINT project_pages_project_id_376ba35a_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: project_pages project_pages_updated_by_id_b80bf0f4_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_pages
    ADD CONSTRAINT project_pages_updated_by_id_b80bf0f4_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: project_pages project_pages_workspace_id_13ed9e73_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_pages
    ADD CONSTRAINT project_pages_workspace_id_13ed9e73_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: projects project_project_lead_id_caf8e353_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT project_project_lead_id_caf8e353_fk_user_id FOREIGN KEY (project_lead_id) REFERENCES public.users(id);


--
-- Name: project_public_members project_public_members_created_by_id_c4c7c776_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_public_members
    ADD CONSTRAINT project_public_members_created_by_id_c4c7c776_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: project_public_members project_public_members_member_id_52f257f9_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_public_members
    ADD CONSTRAINT project_public_members_member_id_52f257f9_fk_users_id FOREIGN KEY (member_id) REFERENCES public.users(id);


--
-- Name: project_public_members project_public_members_project_id_2dfd893d_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_public_members
    ADD CONSTRAINT project_public_members_project_id_2dfd893d_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: project_public_members project_public_members_updated_by_id_c3e4d675_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_public_members
    ADD CONSTRAINT project_public_members_updated_by_id_c3e4d675_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: project_public_members project_public_members_workspace_id_ebfce110_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_public_members
    ADD CONSTRAINT project_public_members_workspace_id_ebfce110_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: projects project_updated_by_id_fe290525_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT project_updated_by_id_fe290525_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: project_webhooks project_webhooks_created_by_id_c3e4bfa3_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_webhooks
    ADD CONSTRAINT project_webhooks_created_by_id_c3e4bfa3_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: project_webhooks project_webhooks_project_id_bec3cf8c_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_webhooks
    ADD CONSTRAINT project_webhooks_project_id_bec3cf8c_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: project_webhooks project_webhooks_updated_by_id_a0183aeb_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_webhooks
    ADD CONSTRAINT project_webhooks_updated_by_id_a0183aeb_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: project_webhooks project_webhooks_webhook_id_da27c6a7_fk_webhooks_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_webhooks
    ADD CONSTRAINT project_webhooks_webhook_id_da27c6a7_fk_webhooks_id FOREIGN KEY (webhook_id) REFERENCES public.webhooks(id);


--
-- Name: project_webhooks project_webhooks_workspace_id_429ebf05_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.project_webhooks
    ADD CONSTRAINT project_webhooks_workspace_id_429ebf05_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: projects project_workspace_id_01764ff9_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT project_workspace_id_01764ff9_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: projects projects_cover_image_asset_id_e6636b92_fk_file_assets_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_cover_image_asset_id_e6636b92_fk_file_assets_id FOREIGN KEY (cover_image_asset_id) REFERENCES public.file_assets(id);


--
-- Name: projects projects_default_state_id_f13e8b95_fk_states_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_default_state_id_f13e8b95_fk_states_id FOREIGN KEY (default_state_id) REFERENCES public.states(id);


--
-- Name: projects projects_estimate_id_85c7b2ac_fk_estimates_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.projects
    ADD CONSTRAINT projects_estimate_id_85c7b2ac_fk_estimates_id FOREIGN KEY (estimate_id) REFERENCES public.estimates(id);


--
-- Name: slack_project_syncs slack_project_syncs_created_by_id_ec405a17_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.slack_project_syncs
    ADD CONSTRAINT slack_project_syncs_created_by_id_ec405a17_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: slack_project_syncs slack_project_syncs_project_id_016dc792_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.slack_project_syncs
    ADD CONSTRAINT slack_project_syncs_project_id_016dc792_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: slack_project_syncs slack_project_syncs_updated_by_id_152eb3b5_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.slack_project_syncs
    ADD CONSTRAINT slack_project_syncs_updated_by_id_152eb3b5_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: slack_project_syncs slack_project_syncs_workspace_id_d1822b06_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.slack_project_syncs
    ADD CONSTRAINT slack_project_syncs_workspace_id_d1822b06_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: slack_project_syncs slack_project_syncs_workspace_integratio_d89c9b40_fk_workspace; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.slack_project_syncs
    ADD CONSTRAINT slack_project_syncs_workspace_integratio_d89c9b40_fk_workspace FOREIGN KEY (workspace_integration_id) REFERENCES public.workspace_integrations(id);


--
-- Name: social_login_connections social_login_connection_created_by_id_7ca2ef50_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.social_login_connections
    ADD CONSTRAINT social_login_connection_created_by_id_7ca2ef50_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: social_login_connections social_login_connection_updated_by_id_c13deb42_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.social_login_connections
    ADD CONSTRAINT social_login_connection_updated_by_id_c13deb42_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: social_login_connections social_login_connection_user_id_0e26c0c5_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.social_login_connections
    ADD CONSTRAINT social_login_connection_user_id_0e26c0c5_fk_user_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: states state_created_by_id_ff51a50d_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.states
    ADD CONSTRAINT state_created_by_id_ff51a50d_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: states state_project_id_23a65fd6_fk_project_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.states
    ADD CONSTRAINT state_project_id_23a65fd6_fk_project_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: states state_updated_by_id_be298453_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.states
    ADD CONSTRAINT state_updated_by_id_be298453_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: states state_workspace_id_2293282d_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.states
    ADD CONSTRAINT state_workspace_id_2293282d_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: stickies stickies_created_by_id_f72e05c4_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.stickies
    ADD CONSTRAINT stickies_created_by_id_f72e05c4_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: stickies stickies_owner_id_6ee3be2b_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.stickies
    ADD CONSTRAINT stickies_owner_id_6ee3be2b_fk_users_id FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: stickies stickies_updated_by_id_d660f1fb_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.stickies
    ADD CONSTRAINT stickies_updated_by_id_d660f1fb_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: stickies stickies_workspace_id_0094496a_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.stickies
    ADD CONSTRAINT stickies_workspace_id_0094496a_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: teams team_created_by_id_725a9101_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.teams
    ADD CONSTRAINT team_created_by_id_725a9101_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: teams team_updated_by_id_79bb36f2_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.teams
    ADD CONSTRAINT team_updated_by_id_79bb36f2_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: teams team_workspace_id_1d56407f_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.teams
    ADD CONSTRAINT team_workspace_id_1d56407f_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: user_favorites user_favorites_created_by_id_dc025309_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_favorites
    ADD CONSTRAINT user_favorites_created_by_id_dc025309_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: user_favorites user_favorites_parent_id_550512e4_fk_user_favorites_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_favorites
    ADD CONSTRAINT user_favorites_parent_id_550512e4_fk_user_favorites_id FOREIGN KEY (parent_id) REFERENCES public.user_favorites(id);


--
-- Name: user_favorites user_favorites_project_id_359b527f_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_favorites
    ADD CONSTRAINT user_favorites_project_id_359b527f_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: user_favorites user_favorites_updated_by_id_a1a5ac4a_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_favorites
    ADD CONSTRAINT user_favorites_updated_by_id_a1a5ac4a_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: user_favorites user_favorites_user_id_cea7e2d2_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_favorites
    ADD CONSTRAINT user_favorites_user_id_cea7e2d2_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: user_favorites user_favorites_workspace_id_aa90f680_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_favorites
    ADD CONSTRAINT user_favorites_workspace_id_aa90f680_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: user_github_connections user_github_connections_created_by_id_99678dc5_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_github_connections
    ADD CONSTRAINT user_github_connections_created_by_id_99678dc5_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: user_github_connections user_github_connections_updated_by_id_de42cb21_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_github_connections
    ADD CONSTRAINT user_github_connections_updated_by_id_de42cb21_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: user_github_connections user_github_connections_user_id_f0a0dd79_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_github_connections
    ADD CONSTRAINT user_github_connections_user_id_f0a0dd79_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: users_groups user_groups_group_id_b76f8aba_fk_auth_group_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users_groups
    ADD CONSTRAINT user_groups_group_id_b76f8aba_fk_auth_group_id FOREIGN KEY (group_id) REFERENCES public.auth_group(id);


--
-- Name: users_groups user_groups_user_id_abaea130_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users_groups
    ADD CONSTRAINT user_groups_user_id_abaea130_fk_user_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: user_notification_preferences user_notification_pr_created_by_id_54dc743a_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_notification_preferences
    ADD CONSTRAINT user_notification_pr_created_by_id_54dc743a_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: user_notification_preferences user_notification_pr_project_id_e0ca17f8_fk_projects_; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_notification_preferences
    ADD CONSTRAINT user_notification_pr_project_id_e0ca17f8_fk_projects_ FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: user_notification_preferences user_notification_pr_updated_by_id_eb70a86d_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_notification_preferences
    ADD CONSTRAINT user_notification_pr_updated_by_id_eb70a86d_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: user_notification_preferences user_notification_pr_workspace_id_a2321c58_fk_workspace; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_notification_preferences
    ADD CONSTRAINT user_notification_pr_workspace_id_a2321c58_fk_workspace FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: user_notification_preferences user_notification_preferences_user_id_9dccc056_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_notification_preferences
    ADD CONSTRAINT user_notification_preferences_user_id_9dccc056_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: user_recent_visits user_recent_visits_created_by_id_a655b75f_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_recent_visits
    ADD CONSTRAINT user_recent_visits_created_by_id_a655b75f_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: user_recent_visits user_recent_visits_project_id_e5eecf27_fk_projects_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_recent_visits
    ADD CONSTRAINT user_recent_visits_project_id_e5eecf27_fk_projects_id FOREIGN KEY (project_id) REFERENCES public.projects(id);


--
-- Name: user_recent_visits user_recent_visits_updated_by_id_42b12ef2_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_recent_visits
    ADD CONSTRAINT user_recent_visits_updated_by_id_42b12ef2_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: user_recent_visits user_recent_visits_user_id_f5153288_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_recent_visits
    ADD CONSTRAINT user_recent_visits_user_id_f5153288_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: user_recent_visits user_recent_visits_workspace_id_362a4e80_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.user_recent_visits
    ADD CONSTRAINT user_recent_visits_workspace_id_362a4e80_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: users_user_permissions user_user_permission_permission_id_9deb68a3_fk_auth_perm; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users_user_permissions
    ADD CONSTRAINT user_user_permission_permission_id_9deb68a3_fk_auth_perm FOREIGN KEY (permission_id) REFERENCES public.auth_permission(id);


--
-- Name: users_user_permissions user_user_permissions_user_id_ed4a47ea_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users_user_permissions
    ADD CONSTRAINT user_user_permissions_user_id_ed4a47ea_fk_user_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: users users_avatar_asset_id_50fa2043_fk_file_assets_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_avatar_asset_id_50fa2043_fk_file_assets_id FOREIGN KEY (avatar_asset_id) REFERENCES public.file_assets(id);


--
-- Name: users users_cover_image_asset_id_b9679cbc_fk_file_assets_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_cover_image_asset_id_b9679cbc_fk_file_assets_id FOREIGN KEY (cover_image_asset_id) REFERENCES public.file_assets(id);


--
-- Name: webhook_logs webhook_logs_created_by_id_71e7bc38_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.webhook_logs
    ADD CONSTRAINT webhook_logs_created_by_id_71e7bc38_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: webhook_logs webhook_logs_updated_by_id_3d9bad04_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.webhook_logs
    ADD CONSTRAINT webhook_logs_updated_by_id_3d9bad04_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: webhook_logs webhook_logs_workspace_id_ffcd0e31_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.webhook_logs
    ADD CONSTRAINT webhook_logs_workspace_id_ffcd0e31_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: webhooks webhooks_created_by_id_25aca1b0_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.webhooks
    ADD CONSTRAINT webhooks_created_by_id_25aca1b0_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: webhooks webhooks_updated_by_id_ea35154e_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.webhooks
    ADD CONSTRAINT webhooks_updated_by_id_ea35154e_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: webhooks webhooks_workspace_id_da5865d7_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.webhooks
    ADD CONSTRAINT webhooks_workspace_id_da5865d7_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: workspaces workspace_created_by_id_10ad894e_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspaces
    ADD CONSTRAINT workspace_created_by_id_10ad894e_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: workspace_home_preferences workspace_home_prefe_workspace_id_b49f76e0_fk_workspace; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_home_preferences
    ADD CONSTRAINT workspace_home_prefe_workspace_id_b49f76e0_fk_workspace FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: workspace_home_preferences workspace_home_preferences_created_by_id_f31fc163_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_home_preferences
    ADD CONSTRAINT workspace_home_preferences_created_by_id_f31fc163_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: workspace_home_preferences workspace_home_preferences_updated_by_id_14ed118a_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_home_preferences
    ADD CONSTRAINT workspace_home_preferences_updated_by_id_14ed118a_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: workspace_home_preferences workspace_home_preferences_user_id_4087938d_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_home_preferences
    ADD CONSTRAINT workspace_home_preferences_user_id_4087938d_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: workspace_integrations workspace_integratio_integration_id_6cb0aace_fk_integrati; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_integrations
    ADD CONSTRAINT workspace_integratio_integration_id_6cb0aace_fk_integrati FOREIGN KEY (integration_id) REFERENCES public.integrations(id);


--
-- Name: workspace_integrations workspace_integrations_actor_id_21619aa1_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_integrations
    ADD CONSTRAINT workspace_integrations_actor_id_21619aa1_fk_users_id FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: workspace_integrations workspace_integrations_api_token_id_bdb1759b_fk_api_tokens_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_integrations
    ADD CONSTRAINT workspace_integrations_api_token_id_bdb1759b_fk_api_tokens_id FOREIGN KEY (api_token_id) REFERENCES public.api_tokens(id);


--
-- Name: workspace_integrations workspace_integrations_created_by_id_37639c73_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_integrations
    ADD CONSTRAINT workspace_integrations_created_by_id_37639c73_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: workspace_integrations workspace_integrations_updated_by_id_fce01dcb_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_integrations
    ADD CONSTRAINT workspace_integrations_updated_by_id_fce01dcb_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: workspace_integrations workspace_integrations_workspace_id_27ebeb6b_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_integrations
    ADD CONSTRAINT workspace_integrations_workspace_id_27ebeb6b_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: workspace_members workspace_member_created_by_id_8dc8b040_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_member_created_by_id_8dc8b040_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: workspace_member_invites workspace_member_invite_created_by_id_082f21d3_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_member_invites
    ADD CONSTRAINT workspace_member_invite_created_by_id_082f21d3_fk_user_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: workspace_member_invites workspace_member_invite_updated_by_id_d31a9c7f_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_member_invites
    ADD CONSTRAINT workspace_member_invite_updated_by_id_d31a9c7f_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: workspace_member_invites workspace_member_invite_workspace_id_d935b364_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_member_invites
    ADD CONSTRAINT workspace_member_invite_workspace_id_d935b364_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: workspace_members workspace_member_member_id_824f5497_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_member_member_id_824f5497_fk_user_id FOREIGN KEY (member_id) REFERENCES public.users(id);


--
-- Name: workspace_members workspace_member_updated_by_id_1cec0062_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_member_updated_by_id_1cec0062_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: workspace_members workspace_member_workspace_id_33f66d4b_fk_workspace_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_members
    ADD CONSTRAINT workspace_member_workspace_id_33f66d4b_fk_workspace_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: workspaces workspace_owner_id_60a8bafc_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspaces
    ADD CONSTRAINT workspace_owner_id_60a8bafc_fk_user_id FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: workspace_themes workspace_themes_actor_id_0e94172e_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_themes
    ADD CONSTRAINT workspace_themes_actor_id_0e94172e_fk_users_id FOREIGN KEY (actor_id) REFERENCES public.users(id);


--
-- Name: workspace_themes workspace_themes_created_by_id_676e2655_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_themes
    ADD CONSTRAINT workspace_themes_created_by_id_676e2655_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: workspace_themes workspace_themes_updated_by_id_bba863fe_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_themes
    ADD CONSTRAINT workspace_themes_updated_by_id_bba863fe_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: workspace_themes workspace_themes_workspace_id_d1bffad8_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_themes
    ADD CONSTRAINT workspace_themes_workspace_id_d1bffad8_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: workspaces workspace_updated_by_id_09d249ed_fk_user_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspaces
    ADD CONSTRAINT workspace_updated_by_id_09d249ed_fk_user_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: workspace_user_links workspace_user_links_created_by_id_b9ce7a5d_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_links
    ADD CONSTRAINT workspace_user_links_created_by_id_b9ce7a5d_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: workspace_user_links workspace_user_links_owner_id_37d99444_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_links
    ADD CONSTRAINT workspace_user_links_owner_id_37d99444_fk_users_id FOREIGN KEY (owner_id) REFERENCES public.users(id);


--
-- Name: workspace_user_links workspace_user_links_updated_by_id_bd0b017f_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_links
    ADD CONSTRAINT workspace_user_links_updated_by_id_bd0b017f_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: workspace_user_links workspace_user_links_workspace_id_1b0a8e22_fk_workspaces_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_links
    ADD CONSTRAINT workspace_user_links_workspace_id_1b0a8e22_fk_workspaces_id FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: workspace_user_preferences workspace_user_prefe_workspace_id_a345adde_fk_workspace; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_preferences
    ADD CONSTRAINT workspace_user_prefe_workspace_id_a345adde_fk_workspace FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: workspace_user_preferences workspace_user_preferences_created_by_id_2d566570_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_preferences
    ADD CONSTRAINT workspace_user_preferences_created_by_id_2d566570_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: workspace_user_preferences workspace_user_preferences_updated_by_id_65fed266_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_preferences
    ADD CONSTRAINT workspace_user_preferences_updated_by_id_65fed266_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: workspace_user_preferences workspace_user_preferences_user_id_0ba5007a_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_preferences
    ADD CONSTRAINT workspace_user_preferences_user_id_0ba5007a_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: workspace_user_properties workspace_user_prope_workspace_id_1dc3e2a6_fk_workspace; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_properties
    ADD CONSTRAINT workspace_user_prope_workspace_id_1dc3e2a6_fk_workspace FOREIGN KEY (workspace_id) REFERENCES public.workspaces(id);


--
-- Name: workspace_user_properties workspace_user_properties_created_by_id_6d8d1c4e_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_properties
    ADD CONSTRAINT workspace_user_properties_created_by_id_6d8d1c4e_fk_users_id FOREIGN KEY (created_by_id) REFERENCES public.users(id);


--
-- Name: workspace_user_properties workspace_user_properties_updated_by_id_910a2cc5_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_properties
    ADD CONSTRAINT workspace_user_properties_updated_by_id_910a2cc5_fk_users_id FOREIGN KEY (updated_by_id) REFERENCES public.users(id);


--
-- Name: workspace_user_properties workspace_user_properties_user_id_b1079e07_fk_users_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspace_user_properties
    ADD CONSTRAINT workspace_user_properties_user_id_b1079e07_fk_users_id FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: workspaces workspaces_logo_asset_id_a784bb00_fk_file_assets_id; Type: FK CONSTRAINT; Schema: public; Owner: plane
--

ALTER TABLE ONLY public.workspaces
    ADD CONSTRAINT workspaces_logo_asset_id_a784bb00_fk_file_assets_id FOREIGN KEY (logo_asset_id) REFERENCES public.file_assets(id);


--
-- PostgreSQL database dump complete
--

