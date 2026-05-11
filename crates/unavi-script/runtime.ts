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
  "wired:scene/types#[method]mesh.colors",
  "wired:scene/types#[method]mesh.indices",
  "wired:scene/types#[method]mesh.normals",
  "wired:scene/types#[method]mesh.positions",
  "wired:scene/types#[method]mesh.set-colors",
  "wired:scene/types#[method]mesh.set-indices",
  "wired:scene/types#[method]mesh.set-normals",
  "wired:scene/types#[method]mesh.set-positions",
  "wired:scene/types#[method]mesh.set-tangents",
  "wired:scene/types#[method]mesh.set-uv0",
  "wired:scene/types#[method]mesh.set-uv1",
  "wired:scene/types#[method]mesh.tangents",
  "wired:scene/types#[method]mesh.uv0",
  "wired:scene/types#[method]mesh.uv1",
  "wired:scene/types#[method]node.collider",
  "wired:scene/types#[method]node.set-collider",
];

const SCRIPT_ASYNC_EXPORTS = [
  "wired:script/guest-api#init",
  "wired:script/guest-api#render",
  "wired:script/guest-api#tick",
];

export async function instantiateScript(
  bytes: Uint8Array,
  name: string,
  rt: any,
): Promise<any> {
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
  return instance;
}

export async function scriptInit(instance: any): Promise<void> {
  await instance.guestApi.init();
}

export async function scriptRender(instance: any): Promise<void> {
  await instance.guestApi.render();
}

export async function scriptTick(instance: any): Promise<void> {
  await instance.guestApi.tick();
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
