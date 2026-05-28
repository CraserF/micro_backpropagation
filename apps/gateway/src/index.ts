/**
 * Gateway Worker entrypoint.
 *
 * Routes (see PLAN.md §4.4):
 *   POST /v1/chat/completions  - OpenAI-compatible proxy to DeepSeek (SSE), metered & rate-limited
 *   GET  /v1/models            - list available models
 *   ALL  /api/auth/*           - Better Auth (incl. /api/auth/jwks)
 *   ALL  /api/admin/*          - internal ops admin API (role-gated)
 *   GET  /healthz              - liveness
 *
 * Middleware chain on /v1/*: JWT verify -> client-key/tenant scope -> Turnstile
 *   -> rate-limit/quota (Durable Object) -> proxy + meter.
 *
 * This is a Phase 0 stub; see PLAN.md §5 for the build order.
 */
import { Hono } from "hono";

export interface Env {
  DB: D1Database;
  RATE_LIMITER: DurableObjectNamespace;
  DEEPSEEK_API_KEY: string;
  OPENROUTER_API_KEY: string;
  DEEPINFRA_API_KEY: string;
  BETTER_AUTH_SECRET: string;
  TURNSTILE_SECRET_KEY: string;
  PRIMARY_PROVIDER: string;
}

const app = new Hono<{ Bindings: Env }>();

app.get("/healthz", (c) => c.json({ ok: true }));

// TODO Phase 1: mount Better Auth handler at /api/auth/*
// TODO Phase 2: POST /v1/chat/completions -> middleware chain -> inference proxy
// TODO Phase 4: /api/admin/* role-gated ops endpoints

export default app;

// Durable Object: rate limiter / quota counter (implemented in Phase 3).
export { RateLimiter } from "./rate-limiter";
