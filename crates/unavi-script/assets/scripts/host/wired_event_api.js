import { EventReceptor } from "wired:event/types";

const cabiRep = Symbol.for("cabiRep");
const host = globalThis.__unavi_host;

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

export function emit(channel, payload, filter) {
  host.hostEventEmit(scriptId(), channel, payload, {
    nodeRep: filter.node ? filter.node[cabiRep] : undefined,
    radius: filter.scope?.val ?? 0,
    documents: filter.documents ?? undefined,
  });
}

export function listen(channels, filter) {
  const rep = host.hostEventListen(scriptId(), channels, {
    nodeRep: filter.node ? filter.node[cabiRep] : undefined,
    radius: filter.scope?.val ?? 0,
    documents: filter.documents ?? undefined,
  });
  if (!rep) return null;
  const receptor = Object.create(EventReceptor.prototype);
  receptor[cabiRep] = rep;
  return receptor;
}
