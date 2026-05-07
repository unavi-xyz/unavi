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
      localAgent: rt.wired_agent_local_agent.bind(rt),
      localCamera: rt.wired_agent_local_camera.bind(rt),
    },
    "wired:agent/types": {
      Agent: rt.wired_agent_class(),
    },
    "wired:event/api": {
      emit: rt.wired_event_emit.bind(rt),
      listen: rt.wired_event_listen.bind(rt),
    },
    "wired:event/types": {
      EventReceptor: rt.wired_event_receptor_class(),
    },
    "wired:input/api": {
      registerInputListener: rt.wired_input_register_input_listener.bind(rt),
    },
    "wired:input/context": {
      listener: rt.wired_input_context_listener.bind(rt),
    },
    "wired:input/types": {
      InputListener: rt.wired_input_listener_class(),
    },
    "wired:portal/api": {
      listPortals: rt.wired_portal_list_portals.bind(rt),
      openPortal: rt.wired_portal_open_portal.bind(rt),
    },
    "wired:portal/types": {
      Portal: rt.wired_portal_class(),
    },
    "wired:scene/api": {
      createDocument: rt.wired_scene_create_document.bind(rt),
      getDocument: rt.wired_scene_get_document.bind(rt),
      loadHsd: rt.wired_scene_load_hsd.bind(rt),
      removeDocument: rt.wired_scene_remove_document.bind(rt),
      selfDocument: rt.wired_scene_self_document.bind(rt),
      selfNode: rt.wired_scene_self_node.bind(rt),
    },
    "wired:scene/types": {
      Document: rt.wired_scene_doc_class(),
      Material: rt.wired_scene_material_class(),
      Mesh: rt.wired_scene_mesh_class(),
      Node: rt.wired_scene_node_class(),
    },
    "wired:wds/api": {
      getWds: rt.wired_wds_get_wds.bind(rt),
    },
    "wired:wds/types": {
      QueryFuture: rt.wired_query_future_class(),
      ReadFuture: rt.wired_read_future_class(),
      Wds: rt.wired_wds_class(),
    },
  };
}
