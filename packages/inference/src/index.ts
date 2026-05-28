/**
 * Provider-abstraction SDK (PLAN.md §4.1).
 *
 * Single interface over the OpenAI SDK with a configurable baseURL + key, so
 * DeepSeek <-> OpenRouter <-> DeepInfra is a config switch. Failover: on
 * 5xx/timeout from the primary, retry once then fall to the next provider.
 *
 * Phase 0 stub — implemented in Phase 2.
 */
import type { Provider } from "@llm-api/shared";

export interface ProviderConfig {
  provider: Provider;
  baseURL: string;
  apiKey: string;
}

export const PROVIDER_BASE_URLS: Record<Provider, string> = {
  deepseek: "https://api.deepseek.com",
  openrouter: "https://openrouter.ai/api/v1",
  deepinfra: "https://api.deepinfra.com/v1/openai",
};

export interface ChatOptions {
  model: string; // e.g. "deepseek-v4-flash" | "deepseek-v4-pro"
  messages: Array<{ role: string; content: string }>;
  stream?: boolean;
}

export interface ChatClient {
  /** Returns an SSE-compatible stream; records fallback_used on failover. */
  chat(opts: ChatOptions): Promise<Response>;
}

// TODO Phase 2: createChatClient(primary, fallbacks[]) with retry + failover.
