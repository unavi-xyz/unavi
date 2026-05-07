import {
  generate,
  GenerateOptions,
  Transpiled,
} from "@bytecodealliance/jco/component";
import { WASIShim } from "@bytecodealliance/preview2-shim/instantiation";

const SCENE_ASYNC_IMPORTS = [
  "wired:scene/api#create-document",
  "wired:scene/api#get-document",
  "wired:scene/api#load-hsd",
  "wired:scene/api#remove-document",
  "wired:scene/types#[method]document.clone",
  "wired:scene/types#[method]document.create-node",
  "wired:scene/types#[method]document.id",
  "wired:scene/types#[method]document.materials",
  "wired:scene/types#[method]document.meshes",
  "wired:scene/types#[method]document.nodes",
  "wired:scene/types#[method]document.roots",
];

const SCRIPT_ASYNC_EXPORTS = [
  "wired:script/guest-api#init",
  "wired:script/guest-api#render",
  "wired:script/guest-api#tick",
];

export async function build_script(
  bytes: Uint8Array,
  name: string,
  rt: any,
): Promise<void> {
  console.log("Building script", name);

  const options: GenerateOptions = {
    asyncMode: {
      tag: "jspi",
      val: {
        imports: SCENE_ASYNC_IMPORTS,
        exports: SCRIPT_ASYNC_EXPORTS,
      },
    },
    instantiation: { tag: "async" },
    name,
    noNodejsCompat: true,
    noTypescript: true,
    strict: false,
  };

  try {
    const result = await (generate(
      bytes,
      options,
    ) as unknown as Promise<Transpiled>);
    console.log("Generated script", name, result);

    const jsFile = result.files.find(([name]) => name.endsWith(".js"));
    if (jsFile == undefined) {
      console.warn("Transpiled JS not found");
      return;
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

    const wasi = new WASIShim({
      sandbox: {
        preopens: {},
        env: {},
        args: [],
        enableNetwork: false,
      },
    });
    const imports = build_imports(wasi, rt);

    const instance = await mod.instantiate(getCoreModule, imports);
    console.log("Instantiated script", name, instance);

    await instance.guestApi.init();
    console.log("Initialized script", name);

    // TODO send script to Rust -> Bevy calls tick

    await instance.guestApi.tick();
    console.log("Ticked script", name);
  } catch (err) {
    console.error("Failed to build script", err);
  }
}

function build_imports(wasi: WASIShim, rt: any) {
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
      EventReceptor: rt.wiredEventReceptorClass(),
    },
    "wired:input/api": {
      registerInputListener: rt.wiredInputRegisterInputListener.bind(rt),
    },
    "wired:input/context": {
      listener: rt.wiredInputContextListener.bind(rt),
    },
    "wired:input/types": {
      InputListener: rt.wiredInputListenerClass(),
    },
    "wired:portal/api": {
      listPortals: rt.wiredPortalListPortals.bind(rt),
      openPortal: rt.wiredPortalOpenPortal.bind(rt),
    },
    "wired:portal/types": {
      Portal: rt.wiredPortalClass(),
    },
    "wired:scene/api": {
      createDocument: rt.wiredSceneCreateDocument.bind(rt),
      getDocument: rt.wiredSceneGetDocument.bind(rt),
      loadHsd: rt.wiredSceneLoadHsd.bind(rt),
      removeDocument: rt.wiredSceneRemoveDocument.bind(rt),
      selfDocument: rt.wiredSceneSelfDocument.bind(rt),
      selfNode: rt.wiredSceneSelfNode.bind(rt),
    },
    "wired:scene/types": {
      Document: rt.wiredSceneDocClass(),
      Material: rt.wiredSceneMaterialClass(),
      Mesh: rt.wiredSceneMeshClass(),
      Node: rt.wiredSceneNodeClass(),
    },
    "wired:wds/api": {
      getWds: rt.wiredWdsGetWds.bind(rt),
    },
    "wired:wds/types": {
      QueryFuture: rt.wiredQueryFutureClass(),
      ReadFuture: rt.wiredReadFutureClass(),
      Wds: rt.wiredWdsClass(),
    },
  };
}
