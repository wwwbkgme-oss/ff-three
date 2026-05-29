-- 005: Event store tables
-- Append-only log for all domain events.

-- One row per aggregate stream (version = total events appended).
CREATE TABLE IF NOT EXISTS event_streams (
    stream_id  UUID    PRIMARY KEY,
    version    BIGINT  NOT NULL DEFAULT 0
);

-- Append-only event log.
-- global_offset is a BIGSERIAL so insertion order is globally monotonic.
CREATE TABLE IF NOT EXISTS events (
    global_offset  BIGSERIAL   PRIMARY KEY,
    stream_id      UUID        NOT NULL REFERENCES event_streams(stream_id),
    sequence       BIGINT      NOT NULL,    -- 0-based position within stream
    payload        JSONB       NOT NULL,
    recorded_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT events_stream_sequence_unique UNIQUE (stream_id, sequence)
);

CREATE INDEX IF NOT EXISTS events_stream_id_idx
    ON events (stream_id, sequence ASC);

CREATE INDEX IF NOT EXISTS events_global_offset_idx
    ON events (global_offset ASC);
