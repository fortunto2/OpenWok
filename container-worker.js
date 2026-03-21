// Thin Cloudflare Worker that routes all requests to the Container.
// Required by Cloudflare Containers architecture.

import { Container, getContainer } from "@cloudflare/containers";

export class OpenWokNode extends Container {
  defaultPort = 3000;
  sleepAfter = "10m";
  enableInternet = true;

  // Pass environment variables to the container
  get envVars() {
    return {
      DATABASE_PATH: "/app/data/openwok.db",
      PORT: "3000",
      IP: "0.0.0.0",
    };
  }
}

export default {
  async fetch(request, env) {
    // Singleton pattern — one container instance per node
    const container = getContainer(env.OPENWOK_NODE);
    return container.fetch(request);
  },
};
