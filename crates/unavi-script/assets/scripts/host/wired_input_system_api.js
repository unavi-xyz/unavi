import { InputListener } from "wired:input/types";

const cabiRep = Symbol.for("cabiRep");
const host = globalThis.__unavi_host;

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

export function systemInputListener() {
  const rep = host.hostInputSystemListener(scriptId());
  if (!rep) return null;
  const listener = Object.create(InputListener.prototype);
  listener[cabiRep] = rep;
  return listener;
}
