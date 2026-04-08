import { Agent } from "wired:agent/types";
import { Node } from "wired:scene/types";

const cabiRep = Symbol.for("cabiRep");

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

function host() {
  return globalThis.__unavi_host;
}

export function localAgent() {
  const rep = host().hostAgentContextLocalAgent(scriptId());
  const agent = Object.create(Agent.prototype);
  agent[cabiRep] = rep;
  return agent;
}

export function localCamera() {
  const rep = host().hostAgentContextLocalCamera(scriptId());
  const node = Object.create(Node.prototype);
  node[cabiRep] = rep;
  return node;
}
