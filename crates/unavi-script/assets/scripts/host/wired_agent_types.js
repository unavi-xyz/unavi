import { Node } from "wired:scene/types";

const cabiDispose = Symbol.for("cabiDispose");
const cabiRep = Symbol.for("cabiRep");

export class Agent {
  static [cabiDispose](_rep) {}
  bone(_name) {
    const node = Object.create(Node.prototype);
    node[cabiRep] = 0;
    return node;
  }
  drop() {}
}
