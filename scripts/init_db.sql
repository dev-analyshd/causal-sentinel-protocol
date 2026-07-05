-- Causal Sentinel Protocol — TimescaleDB Schema
-- Behavioral time-series storage for the Akashic Index

-- Enable TimescaleDB
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Agent behavioral events
CREATE TABLE IF NOT EXISTS coherence_events (
    time         TIMESTAMPTZ NOT NULL,
    agent_id     TEXT NOT NULL,
    psi          DOUBLE PRECISION NOT NULL,
    delta        DOUBLE PRECISION NOT NULL,
    lambda_t     DOUBLE PRECISION NOT NULL,
    gate_open    BOOLEAN NOT NULL,
    p_t          DOUBLE PRECISION,
    i_t          DOUBLE PRECISION,
    c_t          DOUBLE PRECISION,
    s_t          DOUBLE PRECISION,
    w_t          DOUBLE PRECISION,
    regime       TEXT,
    block_height BIGINT
);
SELECT create_hypertable('coherence_events', 'time', if_not_exists => TRUE);
CREATE INDEX IF NOT EXISTS idx_coherence_agent ON coherence_events (agent_id, time DESC);

-- SILENCE events
CREATE TABLE IF NOT EXISTS silence_events (
    time            TIMESTAMPTZ NOT NULL,
    agent_id        TEXT NOT NULL,
    psi             DOUBLE PRECISION NOT NULL,
    delta           DOUBLE PRECISION NOT NULL,
    gap             DOUBLE PRECISION NOT NULL,
    limiting_plane  TEXT,
    reason          TEXT,
    block_height    BIGINT
);
SELECT create_hypertable('silence_events', 'time', if_not_exists => TRUE);
CREATE INDEX IF NOT EXISTS idx_silence_agent ON silence_events (agent_id, time DESC);

-- ZK credential issuance log
CREATE TABLE IF NOT EXISTS credential_events (
    time           TIMESTAMPTZ NOT NULL,
    agent_id       TEXT NOT NULL,
    circuit_type   TEXT NOT NULL,
    nullifier      TEXT NOT NULL UNIQUE,
    compliance_tier INTEGER NOT NULL,
    expiry_block   BIGINT NOT NULL,
    deploy_hash    TEXT
);
SELECT create_hypertable('credential_events', 'time', if_not_exists => TRUE);

-- Moat Λ(t) history
CREATE TABLE IF NOT EXISTS moat_history (
    time        TIMESTAMPTZ NOT NULL,
    agent_id    TEXT NOT NULL,
    lambda_t    DOUBLE PRECISION NOT NULL,
    tier        INTEGER NOT NULL,
    block_height BIGINT
);
SELECT create_hypertable('moat_history', 'time', if_not_exists => TRUE);
CREATE INDEX IF NOT EXISTS idx_moat_agent ON moat_history (agent_id, time DESC);

-- EpistaticController state log
CREATE TABLE IF NOT EXISTS epistatic_events (
    time              TIMESTAMPTZ NOT NULL,
    el_state          DOUBLE PRECISION NOT NULL,
    threat_level      INTEGER NOT NULL,
    validator_health  INTEGER NOT NULL,
    network_entropy   INTEGER NOT NULL,
    regime            TEXT NOT NULL,
    block_height      BIGINT
);
SELECT create_hypertable('epistatic_events', 'time', if_not_exists => TRUE);

-- Continuous aggregates: hourly coherence stats per agent
CREATE MATERIALIZED VIEW IF NOT EXISTS coherence_hourly
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', time) AS bucket,
    agent_id,
    AVG(psi)      AS avg_psi,
    AVG(lambda_t) AS avg_lambda,
    MAX(lambda_t) AS max_lambda,
    COUNT(*)      AS evaluations,
    SUM(CASE WHEN gate_open THEN 1 ELSE 0 END) AS gates_open,
    SUM(CASE WHEN NOT gate_open THEN 1 ELSE 0 END) AS silences
FROM coherence_events
GROUP BY bucket, agent_id
WITH NO DATA;

SELECT add_continuous_aggregate_policy('coherence_hourly',
    start_offset => INTERVAL '3 hours',
    end_offset   => INTERVAL '1 minute',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);
