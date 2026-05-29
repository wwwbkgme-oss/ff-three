-- 002: Core tables

CREATE TABLE students (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    username         VARCHAR(100) NOT NULL UNIQUE,
    email            VARCHAR(255) NOT NULL UNIQUE,
    xp               INTEGER      NOT NULL DEFAULT 0,
    level            INTEGER      NOT NULL DEFAULT 1,
    current_biome_id UUID,
    enrolled_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    goals            TEXT[]       NOT NULL DEFAULT '{}',
    knowledge_map    JSONB        NOT NULL DEFAULT '{}',
    mentor_id        UUID
);

CREATE TABLE biomes (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name                VARCHAR(200) NOT NULL,
    slug                VARCHAR(100) NOT NULL UNIQUE,
    domain              biome_domain NOT NULL,
    description         TEXT         NOT NULL,
    lore                TEXT         NOT NULL,
    min_difficulty      INTEGER      NOT NULL DEFAULT 1,
    max_difficulty      INTEGER      NOT NULL DEFAULT 10,
    unlock_requirements TEXT[]       NOT NULL DEFAULT '{}',
    state               biome_state  NOT NULL DEFAULT 'enlightened',
    active_students     INTEGER      NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE quests (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    title        VARCHAR(300) NOT NULL,
    description  TEXT         NOT NULL,
    quest_type   quest_type   NOT NULL,
    difficulty   INTEGER      NOT NULL CHECK (difficulty BETWEEN 1 AND 10),
    xp_reward    INTEGER      NOT NULL,
    biome_id     UUID         NOT NULL REFERENCES biomes(id) ON DELETE CASCADE,
    requirements TEXT[]       NOT NULL DEFAULT '{}',
    test_cases   JSONB        NOT NULL DEFAULT '[]',
    status       quest_status NOT NULL DEFAULT 'available',
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE student_quests (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id   UUID         NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    quest_id     UUID         NOT NULL REFERENCES quests(id)   ON DELETE CASCADE,
    status       quest_status NOT NULL DEFAULT 'in_progress',
    attempts     INTEGER      NOT NULL DEFAULT 0,
    started_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    UNIQUE (student_id, quest_id)
);

CREATE TABLE assessments (
    id                  UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id          UUID             NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    quest_id            UUID             NOT NULL REFERENCES quests(id)   ON DELETE CASCADE,
    assessment_type     assessment_type  NOT NULL,
    submission          TEXT             NOT NULL,
    score               DOUBLE PRECISION NOT NULL CHECK (score BETWEEN 0 AND 1),
    passed              BOOLEAN          NOT NULL,
    feedback            TEXT             NOT NULL,
    test_results        JSONB            NOT NULL DEFAULT '[]',
    performance_metrics JSONB            NOT NULL DEFAULT '{}',
    assessed_at         TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE TABLE sandbox_runs (
    id                UUID                 PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id        UUID                 NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    quest_id          UUID                 REFERENCES quests(id),
    language          programming_language NOT NULL,
    code              TEXT                 NOT NULL,
    status            sandbox_status       NOT NULL DEFAULT 'pending',
    stdout            TEXT,
    stderr            TEXT,
    exit_code         INTEGER,
    execution_time_ms BIGINT,
    memory_used_kb    BIGINT,
    security_scan     JSONB                NOT NULL DEFAULT '{}',
    created_at        TIMESTAMPTZ          NOT NULL DEFAULT NOW(),
    completed_at      TIMESTAMPTZ
);

CREATE TABLE study_groups (
    id          UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(200)     NOT NULL,
    goal        TEXT             NOT NULL,
    biome_id    UUID             REFERENCES biomes(id),
    progress    DOUBLE PRECISION NOT NULL DEFAULT 0.0 CHECK (progress BETWEEN 0 AND 1),
    status      group_status     NOT NULL DEFAULT 'active',
    max_members INTEGER          NOT NULL DEFAULT 10,
    created_at  TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE TABLE group_members (
    group_id     UUID             NOT NULL REFERENCES study_groups(id) ON DELETE CASCADE,
    student_id   UUID             NOT NULL REFERENCES students(id)     ON DELETE CASCADE,
    role         VARCHAR(50)      NOT NULL DEFAULT 'contributor',
    contribution DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    joined_at    TIMESTAMPTZ      NOT NULL DEFAULT NOW(),
    PRIMARY KEY (group_id, student_id)
);

CREATE TABLE achievements (
    id               UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id       UUID             NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    achievement_type achievement_type NOT NULL,
    title            VARCHAR(200)     NOT NULL,
    description      TEXT             NOT NULL,
    xp_reward        INTEGER          NOT NULL DEFAULT 0,
    earned_at        TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE TABLE certifications (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    student_id     UUID         NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    path           VARCHAR(100) NOT NULL,
    credential_id  VARCHAR(100) NOT NULL UNIQUE,
    world_seed     VARCHAR(200) NOT NULL,
    mentor_reviews JSONB        NOT NULL DEFAULT '[]',
    issued_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE (student_id, path)
);
