/**
 * Rate-limit / quota interface (PLAN.md §4.2).
 *
 * Thin contract so the gateway never depends on the backing store. Today this is
 * implemented by a Durable Object; swap to Valkey/Postgres later without touching
 * gateway logic (migration trigger: DO charges > ~$30-60/mo).
 *
 * Phase 0 stub — implemented in Phase 3.
 */
export interface RateLimitResult {
  allowed: boolean;
  remaining: number;
  resetAt: number; // epoch ms
}

export interface RateLimitStore {
  /** Charge `cost` units against `key`; returns whether the request is allowed. */
  check(key: string, cost: number): Promise<RateLimitResult>;
}
