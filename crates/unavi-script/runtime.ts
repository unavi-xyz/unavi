import {
  generate,
  GenerateOptions,
  Transpiled,
} from "@bytecodealliance/jco/component";
import { WASIShim } from "@bytecodealliance/preview2-shim/instantiation";

/**
 * Every import `bindings::native`'s linker marks `async`, named the way
 * `jco`'s `asyncMode.jspi.imports` option expects: `<interface>#<name>` for a
 * free function, `<interface>#[method]<resource>.<name>` for a resource
 * method. Everything else lifts as a plain synchronous call.
 *
 * Unversioned: the import *object keys* `buildImports` below returns must be
 * unversioned (confirmed against a transpiled dummy guest of this crate's
 * `shell` world — jco reads `imports['wired:scene/document']`, not
 * `imports['wired:scene/document@0.1.0']`); this list, read only by
 * `asyncMode`'s own matcher, accepts either form, so it stays unversioned
 * too for one convention.
 */
const ASYNC_IMPORTS = [
  "wired:scene/document#open-document",
  "wired:scene/document#create-document",
  "wired:scene/document#copy-document",
  "wired:scene/document#delete-document",
  "wired:scene/document#[method]document.create-prim",
  "wired:scene/document#[method]document.apply",
  "wired:scene/document#[method]document.commit",
  "wired:scene/document#[method]document.place",
  "wired:shading/graph#set-graph",
  "wired:shading/graph#set-overrides",
  "wired:physics/simulation#raycast",
  "wired:physics/simulation#velocity",
  "wired:physics/simulation#set-velocity",
  "wired:physics/simulation#set-force",
  "wired:agent/local#attach",
  "wired:portal/portals#open",
  "wired:portal/portals#pair",
  "wired:portal/portals#travel",
  "unavi:host/node-storage#get",
  "unavi:host/node-storage#list-entries",
];

/** Every export `bindings::native`'s linker marks `async`: the whole of
 * `wired:script/lifecycle`. */
const ASYNC_EXPORTS = [
  "wired:script/lifecycle#init",
  "wired:script/lifecycle#update",
  "wired:script/lifecycle#fixed-update",
];

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

async function compile(bytes: Uint8Array, name: string): Promise<Compiled> {
  const options: GenerateOptions = {
    asyncMode: {
      tag: "jspi",
      val: {
        imports: ASYNC_IMPORTS,
        exports: ASYNC_EXPORTS,
      },
    },
    instantiation: { tag: "async" },
    name,
    noNamespacedExports: true,
    noNodejsCompat: true,
    noTypescript: true,
    strict: true,
    // `init`'s only WIT error is a plain `string`. Without this, a failing
    // `init` throws a `ComponentError` wrapper instead of that string, and
    // `engine::web::tick`'s `err.as_string()` check (which tells a guest's
    // own `init` failure apart from a real trap) would never match.
    noComponentErrorWrapping: true,
  };

  const result = await (generate(
    bytes,
    options,
  ) as unknown as Promise<Transpiled>);

  const jsFile = result.files.find(([path]) => path.endsWith(".js"));
  if (jsFile == undefined) {
    throw new Error("transpiled JS not found");
  }
  const jsCode = new TextDecoder().decode(jsFile[1]);
  const blob = new Blob([jsCode], { type: "text/javascript" });
  const url = URL.createObjectURL(blob);
  let mod: { instantiate: Compiled["instantiate"] };
  try {
    mod = await import(url);
  } finally {
    URL.revokeObjectURL(url);
  }

  const fileMap = new Map(result.files);
  const getCoreModule = async (path: string): Promise<WebAssembly.Module> => {
    const bytes = fileMap.get(path);
    if (!bytes) {
      throw new Error(`missing wasm module: ${path}`);
    }
    return await WebAssembly.compile(bytes as BufferSource);
  };

  return { getCoreModule, instantiate: mod.instantiate };
}

export async function instantiateScript(
  bytes: Uint8Array,
  hash: string,
  name: string,
  rt: unknown,
): Promise<unknown> {
  let compiled = compiledByHash.get(hash);
  if (compiled == undefined) {
    compiled = await compile(bytes, name);
    compiledByHash.set(hash, compiled);
  }

  const wasi = new WASIShim({
    sandbox: {
      preopens: {},
      env: {},
      args: [],
      enableNetwork: false,
    },
  });
  const imports = buildImports(wasi, rt);
  batchOutput(imports, name, rt);

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
 * Replaces the shim's stdout and stderr, which write straight to the
 * console, one call per write and outside the client's log filter.
 *
 * Gathering a run is this side's job because only this side knows when one
 * ends: writes are held until the microtask queue drains, which under JSPI
 * is the end of the guest's synchronous stretch. `blockingFlush` deliberately
 * does not force it — Rust flushes per line, and honouring that would be the
 * behaviour this replaces. The run then goes to `scriptLog`, which is where
 * native output lands too.
 */
function batchOutput(imports: Record<string, any>, name: string, rt: any) {
  for (const [iface, getter, isError] of [
    ["wasi:cli/stdout", "getStdout", false],
    ["wasi:cli/stderr", "getStderr", true],
  ] as const) {
    const entry = imports[iface];
    if (entry == undefined) continue;

    const decoder = new TextDecoder();
    let held = "";
    let scheduled = false;

    const flush = () => {
      scheduled = false;
      const run = held;
      held = "";
      if (run !== "") rt.scriptLog(name, isError, run);
    };

    const stream = new entry.OutputStream({
      write(contents: Uint8Array) {
        held += decoder.decode(contents, { stream: true });
        if (scheduled) return;
        scheduled = true;
        queueMicrotask(flush);
      },
      blockingFlush() {},
      [Symbol.dispose ?? Symbol.for("dispose")]() {
        flush();
      },
    });

    imports[iface] = { ...entry, [getter]: () => stream };
  }
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
      owner: rt.owner.bind(rt),
      holder: rt.holder.bind(rt),
      isOwner: rt.isOwner.bind(rt),
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
      intents: rt.intents.bind(rt),
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
