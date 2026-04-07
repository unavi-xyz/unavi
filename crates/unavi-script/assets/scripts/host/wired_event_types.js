const cabiRep = Symbol.for("cabiRep");
const cabiDispose = Symbol.for("cabiDispose");
const host = globalThis.__unavi_host;

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

export class EventReceptor {
  static [cabiDispose](rep) {
    host.hostEventReceptorDrop(scriptId(), rep);
  }

  poll() {
    return host.hostEventReceptorPoll(scriptId(), this[cabiRep]);
  }

  drop() {
    host.hostEventReceptorDrop(scriptId(), this[cabiRep]);
  }
}
