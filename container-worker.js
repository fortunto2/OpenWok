// Thin Cloudflare Worker that routes all requests to the Container.
// Required by Cloudflare Containers architecture.

export class OpenWokNode {
  constructor(state) {
    this.state = state;
  }

  async fetch(request) {
    // Forward to container on default port (3000)
    return await this.state.container.fetch(request);
  }
}

export default {
  async fetch(request, env) {
    const id = env.OPENWOK_NODE.idFromName("default");
    const stub = env.OPENWOK_NODE.get(id);
    return stub.fetch(request);
  },
};
