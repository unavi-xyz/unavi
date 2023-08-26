import { getDefaultStore } from "jotai";

export class AtomStore {
  get store() {
    return getDefaultStore();
  }

  get: typeof this.store.get = (atom) => this.store.get(atom);
  set: typeof this.store.set = (atom, ...args) => this.store.set(atom, ...args);
}
