import { generate, Transpiled } from "@bytecodealliance/jco/component";
import { WASIShim } from "@bytecodealliance/preview2-shim/instantiation";
import {
  transpileOptions,
  type TranspileReply,
  type TranspileRequest,
} from "./transpile-options";

interface Compiled {
  getCoreModule: (path: string) => Promise<WebAssembly.Module>;
  instantiate: (
    getCoreModule: (path: string) => Promise<WebAssembly.Module>,
    imports: Record<string, unknown>,
  ) => Promise<unknown>;
}

/**
 * Transpiled output, by the `blake3` hash of the component bytes Rust
 * already hashes its `Wasm` asset by. Two scripts built from the same bytes
 * transpile once; every further instance only instantiates.
 */
const compiledByHash = new Map<string, Compiled>();

/** Collapses concurrent compiles of one hash onto a single transpile. */
const compilingByHash = new Map<string, Promise<Compiled>>();

/**
 * The page's one worker, created on first compile. A worker that fails to
 * load is left undefined and every later compile takes the in-page path.
 */
let transpileWorker: Worker | undefined;
const pendingReplies = new Map<string, (reply: TranspileReply) => void>();

function ensureWorker(): Worker | undefined {
  if (transpileWorker != undefined) {
    return transpileWorker;
  }
  try {
    const created = new Worker(
      new URL("./transpile-worker.js", import.meta.url),
      { type: "module", name: "transpile" },
    );
    created.addEventListener(
      "message",
      (event: MessageEvent<TranspileReply>) => {
        const reply = event.data;
        const settle = pendingReplies.get(reply.hash);
        if (settle == undefined) {
          return;
        }
        pendingReplies.delete(reply.hash);
        settle(reply);
      },
    );
    // A worker that fails to load or to post a cloneable reply never answers,
    // so this fails every request it was holding. The next compile retries
    // through `ensureWorker`.
    const fail = () => {
      const failed = transpileWorker;
      transpileWorker = undefined;
      failed?.terminate();
      for (const [hash, settle] of pendingReplies) {
        pendingReplies.delete(hash);
        settle({ hash, error: "transpile worker failed" });
      }
    };
    created.addEventListener("error", fail);
    created.addEventListener("messageerror", fail);
    transpileWorker = created;
    return created;
  } catch {
    transpileWorker = undefined;
    return undefined;
  }
}

/**
 * Starts the transpile worker so its bundle, which imports jco, downloads and
 * evaluates before the first script compiles. A worker that cannot start
 * leaves the in-page path to every compile, as it would without this.
 */
export function prewarm(): void {
  ensureWorker();
}

async function compileWithWorker(
  worker: Worker,
  bytes: Uint8Array,
  hash: string,
  name: string,
): Promise<Compiled> {
  const reply = await new Promise<TranspileReply>((resolve, reject) => {
    pendingReplies.set(hash, resolve);
    try {
      worker.postMessage({ bytes, hash, name } satisfies TranspileRequest);
    } catch (err) {
      pendingReplies.delete(hash);
      reject(err);
    }
  });
  if ("error" in reply) {
    throw new Error(reply.error);
  }

  const modules = new Map(reply.modules);
  return await assemble(reply.js, async (path) => {
    const module = modules.get(path);
    if (module == undefined) {
      throw new Error(`missing wasm module: ${path}`);
    }
    return module;
  });
}

/**
 * Compiles in page, the fallback `compile` takes when no worker is
 * available. The component bytes are transpiled here through jco.
 */
async function compileInPage(
  bytes: Uint8Array,
  name: string,
): Promise<Compiled> {
  const result = await (generate(
    bytes,
    transpileOptions(name),
  ) as unknown as Promise<Transpiled>);

  const jsFile = result.files.find(([path]) => path.endsWith(".js"));
  if (jsFile == undefined) {
    throw new Error("transpiled JS not found");
  }
  const js = new TextDecoder().decode(jsFile[1]);
  const fileMap = new Map(result.files);
  return await assemble(js, async (path) => {
    const contents = fileMap.get(path);
    if (!contents) {
      throw new Error(`missing wasm module: ${path}`);
    }
    return await WebAssembly.compile(contents as BufferSource);
  });
}

/**
 * A `Compiled` from generated module text and a way to reach its core
 * modules. The text is imported from a blob URL and so must be
 * self-contained, which `instantiation: { tag: "async" }` gives. Core-module
 * loading is left to the `getCoreModule` passed in at instantiation.
 */
async function assemble(
  js: string,
  getCoreModule: Compiled["getCoreModule"],
): Promise<Compiled> {
  const blob = new Blob([js], { type: "text/javascript" });
  const url = URL.createObjectURL(blob);
  let mod: { instantiate: Compiled["instantiate"] };
  try {
    mod = await import(url);
  } finally {
    URL.revokeObjectURL(url);
  }
  return { getCoreModule, instantiate: mod.instantiate };
}

async function compile(
  bytes: Uint8Array,
  hash: string,
  name: string,
): Promise<Compiled> {
  const cached = compiledByHash.get(hash);
  if (cached != undefined) {
    return cached;
  }
  const compiling = compilingByHash.get(hash);
  if (compiling != undefined) {
    return await compiling;
  }

  const pending = compileUncached(bytes, hash, name);
  compilingByHash.set(hash, pending);
  try {
    const compiled = await pending;
    compiledByHash.set(hash, compiled);
    return compiled;
  } finally {
    compilingByHash.delete(hash);
  }
}

async function compileUncached(
  bytes: Uint8Array,
  hash: string,
  name: string,
): Promise<Compiled> {
  const active = ensureWorker();
  if (active != undefined) {
    try {
      return await compileWithWorker(active, bytes, hash, name);
    } catch (err) {
      console.warn(
        `transpile worker failed for ${name}; compiling in page`,
        err,
      );
    }
  }
  return await compileInPage(bytes, name);
}

export async function instantiateScript(
  bytes: Uint8Array,
  hash: string,
  name: string,
  rt: unknown,
): Promise<unknown> {
  const compiled = await compile(bytes, hash, name);

  const wasi = new WASIShim({
    sandbox: {
      preopens: {},
      env: {},
      args: [],
      enableNetwork: false,
    },
    stdout: outputHandler(name, false, rt),
    stderr: outputHandler(name, true, rt),
  });
  const imports = buildImports(wasi, rt);

  return await compiled.instantiate(compiled.getCoreModule, imports);
}

/**
 * `noNamespacedExports` drops the package prefix but keeps each export
 * grouped under its interface's own camelCased name, so `wired:script/
 * lifecycle`'s exports land on `instance.lifecycle`, not on `instance`
 * itself.
 */
interface Lifecycle {
  init(): Promise<void>;
  update(tick: unknown): Promise<void>;
  fixedUpdate(tick: unknown): Promise<void>;
}

function lifecycle(instance: unknown): Lifecycle {
  return (instance as { lifecycle: Lifecycle }).lifecycle;
}

export async function scriptInit(instance: unknown): Promise<void> {
  await lifecycle(instance).init();
}

export async function scriptUpdate(
  instance: unknown,
  tick: unknown,
): Promise<void> {
  await lifecycle(instance).update(tick);
}

export async function scriptFixedUpdate(
  instance: unknown,
  tick: unknown,
): Promise<void> {
  await lifecycle(instance).fixedUpdate(tick);
}

/**
 * The shim's stdout and stderr handler. The defaults write straight to the
 * console, one call per write and outside the app's log filter.
 *
 * Gathering a run is this side's job because only this side knows when one
 * ends: writes are held until the microtask queue drains, which under JSPI
 * is the end of the guest's synchronous stretch. `blockingFlush` deliberately
 * does not force it — Rust flushes per line, and honouring that would be the
 * behaviour this replaces. The run then goes to `scriptLog`, which is where
 * native output lands too.
 */
function outputHandler(name: string, isError: boolean, rt: any) {
  const decoder = new TextDecoder();
  let held = "";
  let scheduled = false;

  const flush = () => {
    scheduled = false;
    const run = held;
    held = "";
    if (run !== "") rt.scriptLog(name, isError, run);
  };

  return {
    write(contents: Uint8Array) {
      held += decoder.decode(contents, { stream: true });
      if (scheduled) return;
      scheduled = true;
      queueMicrotask(flush);
    },
    blockingFlush() {},
    drop() {
      flush();
    },
  };
}

/**
 * The `#[wasm_bindgen]` classes `bindings::web`'s resources are instances
 * of. Trunk's own bootstrap snippet assigns `window.wasmBindings` to the
 * glue module's full namespace unconditionally (confirmed in a built
 * `dist/index.html`), so these are reachable without any reflection trick —
 * jco needs the exact class objects to validate a captured resource with
 * `instanceof` before calling one of its own methods, and this is the only
 * route from Rust's exported classes to this separately bundled module.
 */
function resourceClasses() {
  const bindings = (globalThis as any).wasmBindings;
  if (bindings == undefined) {
    throw new Error(
      "window.wasmBindings is not set; Trunk's bootstrap must run before any script instantiates",
    );
  }
  return bindings;
}

/**
 * Every host import, bound to `rt` (a `bindings::web::Runtime`). A resource
 * import needs its class for jco's `instanceof` validation; dropping one
 * needs nothing here at all — jco's own drop trampoline calls
 * `rsc[Symbol.dispose]()` directly on the captured value, which
 * `wasm-bindgen` already wires to `free()` for every exported class.
 */
function buildImports(wasi: WASIShim, rt: any) {
  const classes = resourceClasses();
  return {
    ...wasi.getImportObject(),
    "wired:core/time": {
      utcOffset: rt.utcOffset.bind(rt),
    },
    "wired:script/host": {
      granted: rt.granted.bind(rt),
    },
    "wired:scene/document": {
      Document: classes.DocumentHandle,
      scriptDocument: rt.scriptDocument.bind(rt),
      scriptPrim: rt.scriptPrim.bind(rt),
      openDocument: rt.openDocument.bind(rt),
      createDocument: rt.createDocument.bind(rt),
      copyDocument: rt.copyDocument.bind(rt),
      deleteDocument: rt.deleteDocument.bind(rt),
    },
    "wired:event/messaging": {
      MessageSubscription: classes.MessageSubscriptionHandle,
      emit: rt.emit.bind(rt),
      listen: rt.listen.bind(rt),
    },
    "wired:input/types": {
      InputSubscription: classes.InputSubscriptionHandle,
    },
    "wired:input/targeted": {
      listen: rt.inputTargetedListen.bind(rt),
    },
    "wired:input/device": {
      listen: rt.inputDeviceListen.bind(rt),
      pointers: rt.inputDevicePointers.bind(rt),
    },
    "wired:physics/simulation": {
      raycast: rt.raycast.bind(rt),
      velocity: rt.velocity.bind(rt),
      setVelocity: rt.setVelocity.bind(rt),
      setForce: rt.setForce.bind(rt),
    },
    "wired:peer/identity": {
      selfDid: rt.selfDid.bind(rt),
    },
    "wired:peer/authority": {
      author: rt.author.bind(rt),
      holder: rt.holder.bind(rt),
      isAuthor: rt.isAuthor.bind(rt),
      isHolder: rt.isHolder.bind(rt),
      takeHold: rt.takeHold.bind(rt),
      releaseHold: rt.releaseHold.bind(rt),
    },
    "wired:agent/local": {
      cameraTransform: rt.cameraTransform.bind(rt),
      boneTransform: rt.boneTransform.bind(rt),
      attach: rt.attach.bind(rt),
    },
    "wired:portal/portals": {
      IntentSubscription: classes.IntentSubscriptionHandle,
      open: rt.open.bind(rt),
      pair: rt.pair.bind(rt),
      travel: rt.travel.bind(rt),
      currentSpace: rt.currentSpace.bind(rt),
      intents: rt.intents.bind(rt),
    },
    "wired:perception/perception": {
      portals: rt.perceptionPortals.bind(rt),
      agents: rt.perceptionAgents.bind(rt),
      profile: rt.perceptionProfile.bind(rt),
      meta: rt.perceptionMeta.bind(rt),
      rung: rt.perceptionRung.bind(rt),
    },
    "wired:shading/graph": {
      setGraph: rt.setGraph.bind(rt),
      overrides: rt.overrides.bind(rt),
      setOverrides: rt.setOverrides.bind(rt),
    },
    "unavi:host/node-storage": {
      PendingValue: classes.PendingValueHandle,
      PendingEntries: classes.PendingEntriesHandle,
      rootDocument: rt.rootDocument.bind(rt),
      registries: rt.registries.bind(rt),
      get: rt.get.bind(rt),
      listEntries: rt.listEntries.bind(rt),
    },
  };
}
