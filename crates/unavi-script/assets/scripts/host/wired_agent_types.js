import { Node } from "wired:scene/types";

const cabiDispose = Symbol.for("cabiDispose");
const cabiRep = Symbol.for("cabiRep");

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

function host() {
  return globalThis.__unavi_host;
}

export class Agent {
  static [cabiDispose](_rep) {}
  bone(name) {
    const rep = host().hostAgentBone(scriptId(), this[cabiRep], name);
    if (rep === null || rep === undefined || rep === 0) return undefined;
    const node = Object.create(Node.prototype);
    node[cabiRep] = rep;
    return node;
  }
  drop() {}
}
