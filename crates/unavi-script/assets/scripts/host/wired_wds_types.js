const cabiDispose = Symbol.for("cabiDispose");
const cabiRep = Symbol.for("cabiRep");

function host() {
  return globalThis.__unavi_host;
}

function scriptId() {
  return globalThis.__unavi_current_script_id;
}

export class Wds {
  static [cabiDispose](rep) {
    host()?.hostWdsDrop?.(scriptId(), rep);
  }

  query(filter) {
    const rep = host().hostWdsQuery(scriptId(), this[cabiRep], filter);
    if (!rep) return null;
    const future = Object.create(QueryFuture.prototype);
    future[cabiRep] = rep;
    return future;
  }

  read(recordId) {
    const rep = host().hostWdsRead(scriptId(), this[cabiRep], recordId);
    if (!rep) return null;
    const future = Object.create(ReadFuture.prototype);
    future[cabiRep] = rep;
    return future;
  }

  drop() {
    Wds[cabiDispose](this[cabiRep]);
  }
}

export class QueryFuture {
  static [cabiDispose](rep) {
    host()?.hostWdsQueryFutureDrop?.(scriptId(), rep);
  }

  poll() {
    return host().hostWdsQueryFuturePoll(scriptId(), this[cabiRep]);
  }

  drop() {
    QueryFuture[cabiDispose](this[cabiRep]);
  }
}

export class ReadFuture {
  static [cabiDispose](rep) {
    host()?.hostWdsReadFutureDrop?.(scriptId(), rep);
  }

  poll() {
    return host().hostWdsReadFuturePoll(scriptId(), this[cabiRep]);
  }

  drop() {
    ReadFuture[cabiDispose](this[cabiRep]);
  }
}
