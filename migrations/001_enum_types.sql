-- 001: PostgreSQL ENUM types
-- Values match sqlx `rename_all = "snake_case"` on Rust enums.

CREATE TYPE quest_type AS ENUM (
    'exploration','construction','research','teaching','combat');
CREATE TYPE quest_status AS ENUM (
    'available','in_progress','completed','failed','locked');
CREATE TYPE biome_domain AS ENUM (
    'algorithms','security','artificial_intelligence',
    'systems','languages','web','data_science','mathematics');
CREATE TYPE biome_state AS ENUM (
    'enlightened','clouded','confused','mastered');
CREATE TYPE sandbox_status AS ENUM (
    'pending','running','completed','failed','timeout','security_violation');
CREATE TYPE programming_language AS ENUM (
    'rust','python','java_script','type_script','go','java','cpp');
CREATE TYPE assessment_type AS ENUM (
    'theory','practice','application','teaching');
CREATE TYPE group_status AS ENUM (
    'active','completed','disbanded');
CREATE TYPE achievement_type AS ENUM (
    'quest_completed','biome_unlocked','structure_built',
    'peer_teaching_session','level_up','certification_earned',
    'first_blood','perfectionist','collaborator');
