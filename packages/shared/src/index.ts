/** Shared domain types (PLAN.md §3). Phase 0 stub. */

export type Provider = "deepseek" | "openrouter" | "deepinfra";

export interface JwtPayload {
  id: string;
  email: string;
  role: "user" | "admin";
  tenantId: string; // org id
  projectId: string;
  plan: string;
}

export interface Project {
  id: string;
  orgId: string;
  name: string;
  status: "active" | "suspended";
  createdAt: string;
}

export interface UsageDaily {
  projectId: string;
  day: string; // YYYY-MM-DD
  model: string;
  promptTokens: number;
  completionTokens: number;
  requests: number;
  costMicros: number;
}
