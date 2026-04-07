import { Document, Node } from "wired:scene/types";

const cabiRep = Symbol.for("cabiRep");

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

function host() {
  return globalThis.__unavi_host;
}

export function selfNode() {
  const rep = host().hostSceneContextSelfNode(scriptId());
  if (!rep) return null;
  const node = Object.create(Node.prototype);
  node[cabiRep] = rep;
  return node;
}

export function selfDocument() {
  const rep = host().hostSceneContextSelfDocument(scriptId());
  const doc = Object.create(Document.prototype);
  doc[cabiRep] = rep;
  return doc;
}

export function getDocument(id) {
  const rep = host().hostSceneContextGetDocument(scriptId(), id);
  if (!rep) return null;
  const doc = Object.create(Document.prototype);
  doc[cabiRep] = rep;
  return doc;
}
