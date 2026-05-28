# MVP Plan — Client-Side LLM API Service ("Firebase for LLM calls")

> Status: **Plan / pre-build**. This document is the implementation contract for the MVP.
> Derived from the technical research spec (DeepSeek-first, Cloudflare-native).
> Date: 2026-05-28.

## 0. What we're building (one paragraph)

A hosted, **client-side-friendly** LLM API gateway. A developer signs up, creates a
project, and embeds a **public client key** in their browser/mobile app — exactly like
Firebase. The client key + a short-lived user **JWT** authorize requests straight from the
client to our edge API, which proxies to **DeepSeek** (primary) with OpenRouter/DeepInfra
failover, while we **rate-limit, meter, and quota** every call. We ship two web surfaces: a
**customer dashboard** (self-serve account, keys, usage) and an **internal ops admin**.

### Product decisions for this MVP

| Decision | Choice | Rationale |
|---|---|---|
| Admin surfaces | **Both** — customer dashboard + internal ops admin | User requested both |
| Billing | **Deferred** — build billing-ready metering, no Stripe yet | Ship auth/keys/usage first; wire payments in Phase 5 |
| Repo strategy | **Fresh start** — new TS product; archive Rust backprop code | Stack is TS/Workers, unrelated to existing Rust |
| Primary inference | **DeepSeek first-party API** | Cheapest for V3/V4; OpenAI-compatible |
| Auth | **Better Auth** (JWT + API key + organization plugins) | TS-native, self-hosted, Firebase-style JWKS |
| Hosting | **Cloudflare Workers + Durable Objects + D1** | $5/mo, no egress, SSE, 0→10k no cold start |
| Bot defense | **Cloudflare Turnstile** | Free App Check analog |

---

## 1. Architecture overview

```
                         ┌─────────────────────────────────────────────┐
  Customer's client app  │            Cloudflare Workers (edge)         │
  (browser / mobile)     │                                              │
  ─ public client key ──▶ │  /v1/chat/completions  (gateway)            │
  ─ user JWT (Bearer) ──▶ │    1. verify JWT via JWKS (jose)            │
                         │    2. validate client key + Turnstile token  │
                         │    3. rate-limit + quota  (Durable Object)   │
                         │    4. proxy → DeepSeek (SSE stream)          │──▶ DeepSeek API
                         │    5. meter tokens → usage rollup (D1)       │     (OpenRouter/
                         │                                              │      DeepInfra failover)
  Dashboard / Admin  ───▶ │  /api/auth/*  (Better Auth)                 │
  (Next.js)              │  /api/admin/* (ops)                          │
                         └───────────────┬──────────────────────────────┘
                                         │
                            D1 (SQLite): users, orgs, projects,
                            api_keys, usage_daily, plans, audit_log
                            Durable Objects: live rate counters / windows
```

**Two deployables, one repo (monorepo):**
1. `apps/gateway` — Cloudflare Worker: the API gateway + Better Auth server + admin API.
2. `apps/web` — Next.js app: customer dashboard + internal ops admin (deploys to Cloudflare Pages).
3. `packages/*` — shared types, the provider-abstraction SDK, the rate-limit interface.

---

## 2. The Firebase-style auth flow (the core of the MVP)

The Firebase pattern maps cleanly onto Better Auth. **Two credentials travel together** on
every client-side LLM request:

| Firebase concept | Our implementation | Sent as |
|---|---|---|
| App API key (public, embeddable) | Better Auth **API key** scoped to a project | `X-Client-Key` header |
| App Check attestation | **Turnstile** token (proves real browser) | `X-Turnstile-Token` header (on token mint) |
| User ID token (short-lived JWT) | Better Auth **JWT plugin** (RS256, 15-min TTL) | `Authorization: Bearer <jwt>` |
| Public key verification | `/api/auth/jwks` JWKS endpoint | edge verifies statelessly |
| Token refresh w/ rotation | Better Auth session refresh | refresh endpoint |

### Request lifecycle at the edge
1. **Extract** `Authorization: Bearer <jwt>`, `X-Client-Key`, optional `X-Turnstile-Token`.
2. **Verify JWT** with `jose` + `createRemoteJWKSet('/api/auth/jwks')`. Cache JWKS in
   Worker memory; refetch on unknown `kid` (handles rotation/grace period).
3. **Validate client key**: look up in D1 (cached), confirm it's active and belongs to the
   same `tenantId`/`projectId` claimed in the JWT payload. Reject mismatches.
4. **Turnstile** (on public/anonymous-leaning routes and on token mint): verify token
   server-side (single-use, 300s validity).
5. **Rate-limit + quota** (Durable Object keyed by `projectId`): sliding window + monthly
   quota check. 429 with `Retry-After` on breach.
6. **Proxy** to DeepSeek; stream SSE back unchanged.
7. **Meter** prompt/completion tokens from the response; increment DO counter and enqueue a
   daily rollup into D1 `usage_daily`.

### JWT payload (`definePayload`)
```ts
{ id, email, role, tenantId /* org id */, projectId, plan }
```
> Known gap from research: org/tenant-scoped API keys are a Better Auth feature request
> (issue #4746). **Workaround:** store `tenantId`/`projectId` in API-key **metadata** and
> enforce the scope ourselves in the gateway middleware until upstream lands it.

---

## 3. Data model (D1 / SQLite)

Better Auth owns `user`, `session`, `account`, `verification`, `jwks`, `apikey`,
`organization`, `member`, `invitation`. We add:

```sql
-- A project = a Firebase "app". Holds embeddable client keys.
project        (id, org_id, name, created_at, status)
-- Plans / limits (billing-ready, no Stripe yet)
plan           (id, name, monthly_token_quota, rps_limit, price_cents_month)
org_plan       (org_id, plan_id, status, current_period_start, current_period_end)
-- Usage rollup (DO holds the live counter; this is the durable history)
usage_daily    (id, project_id, day, model, prompt_tokens, completion_tokens, requests, cost_micros)
-- Ops + security
audit_log      (id, actor_id, action, target, metadata_json, created_at)
provider_event (id, project_id, provider, status, latency_ms, fallback_used, created_at)
```

`api_key` rows carry `metadata = { tenantId, projectId, type: "publishable" }`. Publishable
keys are **public** (embeddable) and rate-limited; we never issue secret keys for client apps.

---

## 4. Component plan

### 4.1 Inference provider abstraction (`packages/inference`)
- Single interface: `chat(stream, model, messages, opts)` returning an SSE-compatible stream.
- Implemented over the **OpenAI SDK** with a configurable `baseURL` + key, so
  **DeepSeek ↔ OpenRouter ↔ DeepInfra is a config switch**.
- Model routing: `deepseek-v4-flash` for general traffic, `deepseek-v4-pro` (thinking) for
  reasoning routes. Aliases `deepseek-chat`/`deepseek-reasoner` accepted.
- **Failover policy**: on 5xx/timeout/peak-overload from DeepSeek, retry once then fall to
  OpenRouter (DeepInfra as third). Record `fallback_used` in `provider_event`.
- ⚠️ Budget for V4-Pro **full price** ($1.74/$3.48) — the 75% promo expires 2026-05-31.

### 4.2 Rate limiting + quota (`packages/ratelimit` + Durable Object)
- Thin interface `check(key, cost) → {allowed, remaining, resetAt}` so we can swap DO →
  Valkey/Postgres later without touching gateway logic.
- DO implements a **sliding-window RPS limiter** + **monthly token quota** per project.
- Daily flush of DO counters → `usage_daily` for durable accounting and dashboard charts.
- Migration trigger: if DO request+duration charges exceed ~$30–60/mo, move counters to
  self-hosted Valkey on a Hetzner box.

### 4.3 Auth server (`apps/gateway`, Better Auth)
- Plugins: **jwt** (RS256, JWKS, rotation + 30-day grace), **apiKey** (publishable keys),
  **organization** (multi-tenant). Email+password + email verification + bot detection on.
- Endpoints under `/api/auth/*` including `/api/auth/jwks`.
- Short-lived access JWT (15 min) + rotating refresh session.

### 4.4 Gateway Worker (`apps/gateway`)
- Routes: `POST /v1/chat/completions` (OpenAI-compatible, SSE), `/v1/models`, `/healthz`.
- Middleware chain: JWT verify → client-key/tenant scope → Turnstile → rate-limit/quota →
  proxy+meter. Each step short-circuits with a clean OpenAI-style error envelope.
- CORS configured for customer client-side origins (per-project allowlist).

### 4.5 Customer dashboard (`apps/web`, Next.js)
- Sign up / sign in (Better Auth client), email verification.
- Create org + project; **generate/copy/revoke publishable client keys**.
- Usage dashboard: tokens & requests over time (from `usage_daily`), current quota status.
- Quickstart snippet: drop-in client code showing the Firebase-style init + a chat call.
- Settings: allowed origins (CORS), plan (display only until billing lands).

### 4.6 Internal ops admin (`apps/web`, role-gated)
- Gated by `role === "admin"` claim; separate route group `/admin`.
- List all orgs/projects, view usage, **suspend/reactivate** accounts and keys.
- Provider health view (from `provider_event`): latency, error rate, fallback rate.
- Plan/quota editor; audit log viewer.

### 4.7 Client SDK quickstart (deliverable, not a package yet)
- Documented snippet (and optional `packages/client` later) that wraps: init with public
  client key + Turnstile, sign-in to get JWT, auto-refresh, and call `/v1/chat/completions`.

---

## 5. Phased delivery

| Phase | Deliverable | Exit criteria |
|---|---|---|
| **0. Scaffold** | Monorepo (`apps/gateway`, `apps/web`, `packages/*`), Wrangler + D1 + DO bindings, CI. Archive Rust code under `archive/`. | `wrangler dev` runs; `pnpm build` green. |
| **1. Auth core** | Better Auth (jwt+apiKey+organization), JWKS endpoint, sign-up/in in dashboard. | A user can register, get a JWT, fetch JWKS, mint a publishable key. |
| **2. Gateway + inference** | `/v1/chat/completions` proxy to DeepSeek with SSE; provider abstraction + failover. | Streamed completion from a browser using client key + JWT. |
| **3. Rate-limit + metering** | Durable Object limiter + quota; `usage_daily` rollup. | 429 on limit breach; usage rows accrue per request. |
| **4. Dashboards** | Customer dashboard (keys/usage/CORS) + internal ops admin (suspend/usage/health). | Customer self-serves a key; admin suspends an account. |
| **5. (Later) Billing** | Stripe Checkout + Portal + webhooks gating `org_plan.status`. | Subscription state gates API access. |

---

## 6. Cost envelope (beta, 0–1k users)
- Workers Paid: **$5/mo** (DO + D1 included at this scale).
- Better Auth + Turnstile: **$0**.
- DeepSeek inference (~50M tok/mo, V4-Flash): **~$7–15/mo**.
- **Total ≈ $12–20/mo + inference.**

## 7. Migration triggers (when to leave the cheap path)
- Node-only lib incompatible with V8 isolates → move gateway to Hetzner+Coolify or Railway.
- DO charges > ~$30–60/mo → counters to self-hosted Valkey.
- DeepSeek user-visible failures at peak → promote OpenRouter/DeepInfra to primary on route.
- Data-sovereignty need → self-host R1-Distill-Qwen-32B on a 24 GB GPU.
- GUI-managed IdP need → Authentik (Zitadel only for hard multi-tenant isolation).

## 8. Risks / caveats
- **V4-Pro promo expires 2026-05-31** — budget full price.
- **DeepSeek hosted in China/HK** — compliance + reliability reason to keep failover wired.
- **Turnstile efficacy contested** (~33% bot catch claim) — treat as one layer atop
  backend rate limiting, not the only defense; it can block some VPN users.
- **Org-scoped API keys** not yet first-class in Better Auth (#4746) — enforce scope in
  gateway via key metadata until upstream merges.
- **DO SQLite billing started 2026-01-07** — confirm current free allotments before relying
  on "included."
- Publishable keys are **public**; never put secret/inference provider keys client-side —
  the Worker holds the DeepSeek key as a secret binding.

## 9. Open items to confirm before/just-after Phase 0
- Final monorepo tool (pnpm workspaces assumed) and Next.js hosting (Cloudflare Pages assumed).
- Plan tiers + quota numbers for `plan` table (placeholder values in Phase 3).
- Whether to publish `packages/client` SDK now or ship a documented snippet first.
