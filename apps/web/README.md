# web

Next.js app hosting two surfaces (PLAN.md §4.5–4.6):

- **Customer dashboard** — sign up/in, create orgs/projects, generate & revoke publishable
  client keys, view usage & quota, configure allowed CORS origins, copy the quickstart snippet.
- **Internal ops admin** (`/admin`, role-gated) — list all orgs/projects, view usage,
  suspend/reactivate accounts & keys, provider health view, plan/quota editor, audit log.

Scaffolded in Phase 0; built in Phase 1 (auth) and Phase 4 (dashboards).
Planned hosting: Cloudflare Pages.
