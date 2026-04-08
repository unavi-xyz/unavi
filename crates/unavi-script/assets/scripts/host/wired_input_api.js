import { InputListener } from "wired:input/types";

const cabiRep = Symbol.for("cabiRep");

function host() {
  return globalThis.__unavi_host;
}

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

export function registerInputListener(target) {
  const rep = host().hostInputRegisterListener(scriptId(), target[cabiRep]);
  if (!rep) return null;
  const listener = Object.create(InputListener.prototype);
  listener[cabiRep] = rep;
  return listener;
}
