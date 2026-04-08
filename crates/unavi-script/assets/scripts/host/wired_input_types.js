const cabiRep = Symbol.for("cabiRep");
const cabiDispose = Symbol.for("cabiDispose");

function host() {
  return globalThis.__unavi_host;
}

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

export class InputListener {
  static [cabiDispose](rep) {
    host().hostInputListenerDrop(scriptId(), rep);
  }

  poll() {
    return host().hostInputListenerPoll(scriptId(), this[cabiRep]);
  }

  drop() {
    host().hostInputListenerDrop(scriptId(), this[cabiRep]);
  }
}
