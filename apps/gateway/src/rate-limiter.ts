/**
 * RateLimiter Durable Object (PLAN.md §4.2).
 *
 * One instance per project. Holds the live sliding-window RPS counter and the
 * monthly token quota. Counters flush daily into D1 `usage_daily` for durable
 * accounting and dashboard charts.
 *
 * Phase 0 stub — real sliding-window + quota logic lands in Phase 3.
 * Kept behind the @llm-api/ratelimit interface so it can be swapped for
 * Valkey/Postgres later without touching gateway logic.
 */
export class RateLimiter {
  state: DurableObjectState;

  constructor(state: DurableObjectState) {
    this.state = state;
  }

  async fetch(_request: Request): Promise<Response> {
    // TODO Phase 3: check(key, cost) -> { allowed, remaining, resetAt }
    return new Response(JSON.stringify({ allowed: true }), {
      headers: { "content-type": "application/json" },
    });
  }
}
