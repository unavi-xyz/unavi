import { Wds } from "wired:wds/types";

const cabiRep = Symbol.for("cabiRep");
const host = globalThis.__unavi_host;

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

export function wds() {
  const rep = host.hostWdsGetWds(scriptId());
  if (!rep) return null;
  const instance = Object.create(Wds.prototype);
  instance[cabiRep] = rep;
  return instance;
}
