import {
  generate,
  GenerateOptions,
  Transpiled,
} from "@bytecodealliance/jco/component";
import { WASIShim } from "@bytecodealliance/preview2-shim/instantiation";

const SCRIPT_ASYNC_EXPORTS = [
  "wired:script/guest-api#init",
  "wired:script/guest-api#update",
  "wired:script/guest-api#fixed-update",
];

export async function instantiateScript(
  bytes: Uint8Array,
  name: string,
  rt: any,
): Promise<any> {
  console.log("Building script", name);

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

  const options: GenerateOptions = {
    asyncMode: {
      tag: "jspi",
      val: {
        imports: collectAsyncImports(imports),
        exports: SCRIPT_ASYNC_EXPORTS,
      },
    },
    instantiation: { tag: "async" },
    name,
    noNamespacedExports: true,
    noNodejsCompat: true,
    noTypescript: true,
    strict: true,
    // tracing: true,
  };

  const result = await (generate(
    bytes,
    options,
  ) as unknown as Promise<Transpiled>);
  console.log("Generated script", name, result);

  const jsFile = result.files.find(([name]) => name.endsWith(".js"));
  if (jsFile == undefined) {
    throw new Error("Transpiled JS not found");
  }
  const jsCode = new TextDecoder().decode(jsFile[1]);
  const blob = new Blob([jsCode], { type: "text/javascript" });
  const url = URL.createObjectURL(blob);

  const mod = await import(url);

  const fileMap = new Map(result.files);

  async function getCoreModule(path: string): Promise<WebAssembly.Module> {
    const bytes = fileMap.get(path);
    if (!bytes) {
      throw new Error(`Missing wasm module: ${path}`);
    }
    return await WebAssembly.compile(bytes as BufferSource);
  }

  const instance = await mod.instantiate(getCoreModule, imports);
  if (options.tracing) {
    // Only run for a limited number of ticks if we are tracing calls for debugging.
    // Too many starts to lag the browser.
    instance.ticks = 0;
    instance.maxTicks = 2;
  }
  instance.name = name;
  console.log("Instantiated script", name, instance);

  return instance;
}

export async function scriptInit(instance: any): Promise<void> {
  await instance.guestApi.init();
}

export async function scriptUpdate(instance: any): Promise<void> {
  if (instance.ticks !== undefined && instance.ticks >= instance.maxTicks) {
    return;
  }
  await instance.guestApi.update();
}

export async function scriptFixedUpdate(instance: any): Promise<void> {
  if (instance.ticks !== undefined) {
    if (instance.ticks >= instance.maxTicks) {
      return;
    }
    instance.ticks += 1;
  }
  await instance.guestApi.fixedUpdate();
}

/**
 * Replaces the shim's stdout and stderr, which write straight to the console,
 * one call per write and outside the client's log filter.
 *
 * Gathering a run is this side's job because only this side knows when one
 * ends: writes are held until the microtask queue drains, which under JSPI is
 * the end of the guest's synchronous stretch. `blockingFlush` deliberately
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

function buildImports(wasi: WASIShim, rt: any) {
  return {
    ...wasi.getImportObject(),
    "wired:agent/api": {
      localAgent: rt.wiredAgentLocalAgent.bind(rt),
      localCamera: rt.wiredAgentLocalCamera.bind(rt),
    },
    "wired:agent/types": {
      Agent: rt.wiredAgentClass(),
    },
    "wired:event/api": {
      emit: rt.wiredEventEmit.bind(rt),
      listen: rt.wiredEventListen.bind(rt),
    },
    "wired:event/types": {
      Event: rt.wiredEventClass(),
      EventReceptor: rt.wiredEventReceptorClass(),
    },
    "wired:input/api": {
      registerInputListener: rt.wiredInputRegisterInputListener.bind(rt),
    },
    "wired:input/context": {
      registerGlobalInputListener:
        rt.wiredInputRegisterGlobalInputListener.bind(rt),
      pointers: rt.wiredInputPointers.bind(rt),
    },
    "wired:input/types": {
      InputListener: rt.wiredInputListenerClass(),
    },
    "wired:kv/api": {
      selfKv: rt.wiredKvSelfKv.bind(rt),
      getKv: rt.wiredKvGetKv.bind(rt),
    },
    "wired:kv/types": {
      Kv: rt.wiredKvClass(),
    },
    "wired:peer/api": {
      selfPeer: rt.wiredPeerSelfPeer.bind(rt),
      selfDid: rt.wiredPeerSelfDid.bind(rt),
      docOwner: rt.wiredPeerDocOwner.bind(rt),
      isSelfOwner: rt.wiredPeerIsSelfOwner.bind(rt),
    },
    "wired:peer/types": {},
    "wired:physics/api": {
      raycast: rt.wiredPhysicsRaycast.bind(rt),
      getLinearVelocity: rt.wiredPhysicsGetLinearVelocity.bind(rt),
      setLinearVelocity: rt.wiredPhysicsSetLinearVelocity.bind(rt),
      setAngularVelocity: rt.wiredPhysicsSetAngularVelocity.bind(rt),
      applyForce: rt.wiredPhysicsApplyForce.bind(rt),
      claimAuthority: rt.wiredPhysicsClaimAuthority.bind(rt),
      releaseAuthority: rt.wiredPhysicsReleaseAuthority.bind(rt),
    },
    "wired:portal/api": {
      open: rt.wiredPortalOpen.bind(rt),
      travel: rt.wiredPortalTravel.bind(rt),
    },
    "wired:scene/api": {
      createDocument: rt.wiredSceneCreateDocument.bind(rt),
      createDocumentFromPrefab: rt.wiredSceneCreateDocumentFromPrefab.bind(rt),
      getDocument: rt.wiredSceneGetDocument.bind(rt),
      removeDocument: rt.wiredSceneRemoveDocument.bind(rt),
      saveDocument: rt.wiredSceneSaveDocument.bind(rt),
      selfDocument: rt.wiredSceneSelfDocument.bind(rt),
      selfPrim: rt.wiredSceneSelfPrim.bind(rt),
      syncDocument: rt.wiredSceneSyncDocument.bind(rt),
    },
    "wired:scene/types": {
      Document: rt.wiredSceneDocClass(),
      Prim: rt.wiredScenePrimClass(),
    },
    "wired:storage/api": {
      getStorage: rt.wiredStorageGetStorage.bind(rt),
    },
    "wired:storage/types": {
      GetFuture: rt.wiredGetFutureClass(),
      ListFuture: rt.wiredListFutureClass(),
      Storage: rt.wiredStorageClass(),
    },
  };
}

const camelToKebab = (s: string): string =>
  s
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1-$2")
    .toLowerCase();

const isResourceClass = (value: unknown): value is { prototype: object } =>
  typeof value === "function" &&
  (value as { prototype?: object }).prototype != null &&
  Object.getOwnPropertyNames((value as { prototype: object }).prototype).some(
    (n) => n !== "constructor",
  );

function collectAsyncImports(imports: Record<string, unknown>): string[] {
  const out: string[] = [];
  for (const [iface, members] of Object.entries(imports)) {
    if (!iface.startsWith("wired:")) continue;
    for (const [name, value] of Object.entries(
      members as Record<string, unknown>,
    )) {
      if (isResourceClass(value)) {
        const resource = camelToKebab(name);
        const proto = value.prototype;
        for (const method of Object.getOwnPropertyNames(proto)) {
          if (method === "constructor" || method === "free") continue;
          if (method.startsWith("__")) continue;
          const desc = Object.getOwnPropertyDescriptor(proto, method);
          if (!desc || typeof desc.value !== "function") continue;
          out.push(`${iface}#[method]${resource}.${camelToKebab(method)}`);
        }
      } else if (typeof value === "function") {
        out.push(`${iface}#${camelToKebab(name)}`);
      }
    }
  }
  return out;
}
