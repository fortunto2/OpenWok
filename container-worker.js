// Thin Cloudflare Worker that routes all requests to a singleton container.
// Cloudflare Containers beta is more reliable when startup is explicit.

import { Container, getContainer } from "@cloudflare/containers";
import { env } from "cloudflare:workers";

export class OpenWokNode extends Container {
  defaultPort = 3000;
  sleepAfter = "10m";
  enableInternet = true;

  envVars = {
    APP_BASE_URL: env.APP_BASE_URL,
    DATABASE_PATH: "/app/data/openwok.db",
    PORT: "3000",
    IP: "0.0.0.0",
    PUBLIC_APP_URL: env.PUBLIC_APP_URL,
    SUPABASE_ANON_KEY: env.SUPABASE_ANON_KEY,
    SUPABASE_GOOGLE_AUTH_ENABLED: env.SUPABASE_GOOGLE_AUTH_ENABLED,
    SUPABASE_JWT_ISSUER: env.SUPABASE_JWT_ISSUER,
    SUPABASE_URL: env.SUPABASE_URL,
  };
}

export default {
  async fetch(request, env) {
    const versionId = env.CF_VERSION_METADATA?.id ?? "current";
    const container = getContainer(env.OPENWOK_NODE, `singleton-${versionId}`);
    await container.startAndWaitForPorts();
    return container.fetch(request);
  },
};
