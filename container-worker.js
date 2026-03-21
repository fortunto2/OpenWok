// Thin Cloudflare Worker that routes all requests to a singleton container.
// Cloudflare Containers beta is more reliable when startup is explicit.

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
    const container = getContainer(env.OPENWOK_NODE, "singleton");
    await container.startAndWaitForPorts();
    return container.fetch(request);
  },
};
