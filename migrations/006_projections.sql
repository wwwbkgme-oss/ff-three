-- Migration 006: Projection Read Model + Checkpoint Table
--
-- character_views:          materialized CharacterView per NPC
-- projection_checkpoints:   last processed global_offset per projection

CREATE TABLE IF NOT EXISTS character_views (
    id         UUID        PRIMARY KEY,
    data       JSONB       NOT NULL,
    checkpoint BIGINT      NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS projection_checkpoints (
    name   TEXT   PRIMARY KEY,
    offset BIGINT NOT NULL DEFAULT 0
);

-- Index für effiziente Suche nach aktualisierten Views
CREATE INDEX IF NOT EXISTS idx_character_views_checkpoint ON character_views (checkpoint);
