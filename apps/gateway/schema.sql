-- D1 schema (PLAN.md §3). App-owned tables; Better Auth manages its own
-- (user, session, account, verification, jwks, apikey, organization, member, invitation).
-- Phase 0 reference schema — apply with `wrangler d1 execute llm-api --file=schema.sql`.

-- A project = a Firebase "app". Holds embeddable publishable client keys.
CREATE TABLE IF NOT EXISTS project (
  id          TEXT PRIMARY KEY,
  org_id      TEXT NOT NULL,
  name        TEXT NOT NULL,
  status      TEXT NOT NULL DEFAULT 'active', -- active | suspended
  created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Plans / limits (billing-ready; Stripe deferred to Phase 5).
CREATE TABLE IF NOT EXISTS plan (
  id                   TEXT PRIMARY KEY,
  name                 TEXT NOT NULL,
  monthly_token_quota  INTEGER NOT NULL,
  rps_limit            INTEGER NOT NULL,
  price_cents_month    INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS org_plan (
  org_id                TEXT PRIMARY KEY,
  plan_id               TEXT NOT NULL,
  status                TEXT NOT NULL DEFAULT 'active',
  current_period_start  TEXT,
  current_period_end    TEXT
);

-- Durable Object holds the live counter; this is the durable usage history.
CREATE TABLE IF NOT EXISTS usage_daily (
  id                TEXT PRIMARY KEY,
  project_id        TEXT NOT NULL,
  day               TEXT NOT NULL,   -- YYYY-MM-DD
  model             TEXT NOT NULL,
  prompt_tokens     INTEGER NOT NULL DEFAULT 0,
  completion_tokens INTEGER NOT NULL DEFAULT 0,
  requests          INTEGER NOT NULL DEFAULT 0,
  cost_micros       INTEGER NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX IF NOT EXISTS usage_daily_uniq ON usage_daily(project_id, day, model);

-- Ops + security.
CREATE TABLE IF NOT EXISTS audit_log (
  id            TEXT PRIMARY KEY,
  actor_id      TEXT,
  action        TEXT NOT NULL,
  target        TEXT,
  metadata_json TEXT,
  created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS provider_event (
  id            TEXT PRIMARY KEY,
  project_id    TEXT,
  provider      TEXT NOT NULL,
  status        TEXT NOT NULL,
  latency_ms    INTEGER,
  fallback_used INTEGER NOT NULL DEFAULT 0,
  created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
