import { Wds } from "wired:wds/types";

const cabiRep = Symbol.for("cabiRep");
function host() {
  return globalThis.__unavi_host;
}

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

export function getWds() {
  const rep = host().hostWdsGetWds(scriptId());
  if (!rep) return null;
  const instance = Object.create(Wds.prototype);
  instance[cabiRep] = rep;
  return instance;
}
