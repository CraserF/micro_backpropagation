# llm-api-service

A client-side-friendly LLM API gateway — "Firebase for LLM calls". Customers embed a public
client key + a short-lived JWT in their app and call our edge API directly; we rate-limit,
meter, and quota every request while proxying to DeepSeek (with OpenRouter/DeepInfra failover).

See [`PLAN.md`](./PLAN.md) for the full MVP plan and architecture.

## Stack
- **Edge gateway:** Cloudflare Workers + Durable Objects + D1
- **Auth:** Better Auth (JWT/JWKS + API-key + organization plugins)
- **Inference:** DeepSeek first-party API (OpenRouter/DeepInfra failover)
- **Bot defense:** Cloudflare Turnstile
- **Web:** Next.js (customer dashboard + internal ops admin)

## Monorepo layout
```
apps/
  gateway/   Cloudflare Worker: API gateway + Better Auth server + admin API
  web/       Next.js: customer dashboard + internal ops admin
packages/
  shared/    Shared TypeScript types
  inference/ Provider-abstraction SDK (DeepSeek ↔ OpenRouter ↔ DeepInfra)
  ratelimit/ Rate-limit/quota interface (Durable Object today, swappable)
```

## Develop
```bash
pnpm install
pnpm --filter gateway dev   # wrangler dev
pnpm --filter web dev       # next dev
```

## Status
Phase 0 scaffold. See `PLAN.md` §5 for the delivery roadmap.
