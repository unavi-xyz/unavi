import {
  generate,
  GenerateOptions,
  Transpiled,
} from "@bytecodealliance/jco/component";
import { WASIShim } from "@bytecodealliance/preview2-shim/instantiation";

export async function build_script(
  bytes: Uint8Array,
  name: string,
  rt: any,
): Promise<void> {
  console.log("Building script", name);

  const options: GenerateOptions = {
    asyncMode: { tag: "jspi", val: { imports: [], exports: [] } },
    instantiation: { tag: "async" },
    name,
    noNodejsCompat: true,
    noTypescript: true,
    strict: true,
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

    const script = new instance.guestApi.Script();
    console.log("Constructed script", name, script);

    // TODO send script to Rust -> Bevy calls tick

    let res = script.tick();
    console.log("Ticked script", name, res);
  } catch (err) {
    console.error("Failed to build script", err);
  }
}

function build_imports(wasi: WASIShim, rt: any) {
  return {
    ...wasi.getImportObject(),
    "wired:agent/api": {
      localAgent: rt.wired_agent.local_agent,
      localCamera: rt.wired_agent.local_camera,
    },
    "wired:agent/types": {
      Agent: class {
        bone(...a: any[]) { return rt.wired_agent_types.agent_bone(this, ...a); }
      },
    },
    "wired:event/api": {
      emit: rt.wired_event.emit,
      listen: rt.wired_event.listen,
    },
    "wired:event/types": {
      EventReceptor: class {
        poll(...a: any[]) { return rt.wired_event_types.event_receptor_poll(this, ...a); }
      },
    },
    "wired:input/api": {
      registerInputListener: rt.wired_input.register_input_listener,
    },
    "wired:input/context": {
      listener: rt.wired_input_context.listener,
    },
    "wired:input/types": {
      InputListener: class {
        poll(...a: any[]) { return rt.wired_input_types.input_listener_poll(this, ...a); }
      },
    },
    "wired:portal/api": {
      listPortals: rt.wired_portal.list_portals,
      openPortal: rt.wired_portal.open_portal,
    },
    "wired:portal/types": {
      Portal: class {
        close(...a: any[]) { return rt.wired_portal_types.portal_close(this, ...a); }
        destination(...a: any[]) { return rt.wired_portal_types.portal_destination(this, ...a); }
        id(...a: any[]) { return rt.wired_portal_types.portal_id(this, ...a); }
      },
    },
    "wired:scene/api": {
      createDocument: rt.wired_scene.create_document,
      getDocument: rt.wired_scene.get_document,
      loadHsd: rt.wired_scene.load_hsd,
      removeDocument: rt.wired_scene.remove_document,
      selfDocument: rt.wired_scene.self_document,
      selfNode: rt.wired_scene.self_node,
    },
    "wired:scene/types": {
      Document: class {
        addAsset(...a: any[]) { return rt.wired_scene_types.document_add_asset(this, ...a); }
        assets(...a: any[]) { return rt.wired_scene_types.document_assets(this, ...a); }
        clone(...a: any[]) { return rt.wired_scene_types.document_clone(this, ...a); }
        createMaterial(...a: any[]) { return rt.wired_scene_types.document_create_material(this, ...a); }
        createMesh(...a: any[]) { return rt.wired_scene_types.document_create_mesh(this, ...a); }
        createNode(...a: any[]) { return rt.wired_scene_types.document_create_node(this, ...a); }
        globalTransform(...a: any[]) { return rt.wired_scene_types.document_global_transform(this, ...a); }
        id(...a: any[]) { return rt.wired_scene_types.document_id(this, ...a); }
        materials(...a: any[]) { return rt.wired_scene_types.document_materials(this, ...a); }
        meshes(...a: any[]) { return rt.wired_scene_types.document_meshes(this, ...a); }
        nodes(...a: any[]) { return rt.wired_scene_types.document_nodes(this, ...a); }
        public(...a: any[]) { return rt.wired_scene_types.document_public(this, ...a); }
        removeAsset(...a: any[]) { return rt.wired_scene_types.document_remove_asset(this, ...a); }
        removeMaterial(...a: any[]) { return rt.wired_scene_types.document_remove_material(this, ...a); }
        removeMesh(...a: any[]) { return rt.wired_scene_types.document_remove_mesh(this, ...a); }
        removeNode(...a: any[]) { return rt.wired_scene_types.document_remove_node(this, ...a); }
        roots(...a: any[]) { return rt.wired_scene_types.document_roots(this, ...a); }
        rotation(...a: any[]) { return rt.wired_scene_types.document_rotation(this, ...a); }
        scale(...a: any[]) { return rt.wired_scene_types.document_scale(this, ...a); }
        setPublic(...a: any[]) { return rt.wired_scene_types.document_set_public(this, ...a); }
        setRotation(...a: any[]) { return rt.wired_scene_types.document_set_rotation(this, ...a); }
        setScale(...a: any[]) { return rt.wired_scene_types.document_set_scale(this, ...a); }
        setSync(...a: any[]) { return rt.wired_scene_types.document_set_sync(this, ...a); }
        setTransform(...a: any[]) { return rt.wired_scene_types.document_set_transform(this, ...a); }
        setTranslation(...a: any[]) { return rt.wired_scene_types.document_set_translation(this, ...a); }
        sync(...a: any[]) { return rt.wired_scene_types.document_sync(this, ...a); }
        transform(...a: any[]) { return rt.wired_scene_types.document_transform(this, ...a); }
        translation(...a: any[]) { return rt.wired_scene_types.document_translation(this, ...a); }
      },
      Material: class {
        alphaCutoff(...a: any[]) { return rt.wired_scene_types.material_alpha_cutoff(this, ...a); }
        alphaMode(...a: any[]) { return rt.wired_scene_types.material_alpha_mode(this, ...a); }
        baseColor(...a: any[]) { return rt.wired_scene_types.material_base_color(this, ...a); }
        clone(...a: any[]) { return rt.wired_scene_types.material_clone(this, ...a); }
        doubleSided(...a: any[]) { return rt.wired_scene_types.material_double_sided(this, ...a); }
        id(...a: any[]) { return rt.wired_scene_types.material_id(this, ...a); }
        metallic(...a: any[]) { return rt.wired_scene_types.material_metallic(this, ...a); }
        name(...a: any[]) { return rt.wired_scene_types.material_name(this, ...a); }
        roughness(...a: any[]) { return rt.wired_scene_types.material_roughness(this, ...a); }
        setAlphaCutoff(...a: any[]) { return rt.wired_scene_types.material_set_alpha_cutoff(this, ...a); }
        setAlphaMode(...a: any[]) { return rt.wired_scene_types.material_set_alpha_mode(this, ...a); }
        setBaseColor(...a: any[]) { return rt.wired_scene_types.material_set_base_color(this, ...a); }
        setDoubleSided(...a: any[]) { return rt.wired_scene_types.material_set_double_sided(this, ...a); }
        setMetallic(...a: any[]) { return rt.wired_scene_types.material_set_metallic(this, ...a); }
        setName(...a: any[]) { return rt.wired_scene_types.material_set_name(this, ...a); }
        setRoughness(...a: any[]) { return rt.wired_scene_types.material_set_roughness(this, ...a); }
        setSync(...a: any[]) { return rt.wired_scene_types.material_set_sync(this, ...a); }
        setUnlit(...a: any[]) { return rt.wired_scene_types.material_set_unlit(this, ...a); }
        sync(...a: any[]) { return rt.wired_scene_types.material_sync(this, ...a); }
        unlit(...a: any[]) { return rt.wired_scene_types.material_unlit(this, ...a); }
      },
      Mesh: class {
        clone(...a: any[]) { return rt.wired_scene_types.mesh_clone(this, ...a); }
        colors(...a: any[]) { return rt.wired_scene_types.mesh_colors(this, ...a); }
        id(...a: any[]) { return rt.wired_scene_types.mesh_id(this, ...a); }
        indices(...a: any[]) { return rt.wired_scene_types.mesh_indices(this, ...a); }
        name(...a: any[]) { return rt.wired_scene_types.mesh_name(this, ...a); }
        normals(...a: any[]) { return rt.wired_scene_types.mesh_normals(this, ...a); }
        positions(...a: any[]) { return rt.wired_scene_types.mesh_positions(this, ...a); }
        setColors(...a: any[]) { return rt.wired_scene_types.mesh_set_colors(this, ...a); }
        setIndices(...a: any[]) { return rt.wired_scene_types.mesh_set_indices(this, ...a); }
        setName(...a: any[]) { return rt.wired_scene_types.mesh_set_name(this, ...a); }
        setNormals(...a: any[]) { return rt.wired_scene_types.mesh_set_normals(this, ...a); }
        setPositions(...a: any[]) { return rt.wired_scene_types.mesh_set_positions(this, ...a); }
        setSync(...a: any[]) { return rt.wired_scene_types.mesh_set_sync(this, ...a); }
        setTangents(...a: any[]) { return rt.wired_scene_types.mesh_set_tangents(this, ...a); }
        setTopology(...a: any[]) { return rt.wired_scene_types.mesh_set_topology(this, ...a); }
        setUv0(...a: any[]) { return rt.wired_scene_types.mesh_set_uv0(this, ...a); }
        setUv1(...a: any[]) { return rt.wired_scene_types.mesh_set_uv1(this, ...a); }
        sync(...a: any[]) { return rt.wired_scene_types.mesh_sync(this, ...a); }
        tangents(...a: any[]) { return rt.wired_scene_types.mesh_tangents(this, ...a); }
        topology(...a: any[]) { return rt.wired_scene_types.mesh_topology(this, ...a); }
        uv0(...a: any[]) { return rt.wired_scene_types.mesh_uv0(this, ...a); }
        uv1(...a: any[]) { return rt.wired_scene_types.mesh_uv1(this, ...a); }
      },
      Node: class {
        addChild(...a: any[]) { return rt.wired_scene_types.node_add_child(this, ...a); }
        children(...a: any[]) { return rt.wired_scene_types.node_children(this, ...a); }
        clone(...a: any[]) { return rt.wired_scene_types.node_clone(this, ...a); }
        collider(...a: any[]) { return rt.wired_scene_types.node_collider(this, ...a); }
        globalTransform(...a: any[]) { return rt.wired_scene_types.node_global_transform(this, ...a); }
        id(...a: any[]) { return rt.wired_scene_types.node_id(this, ...a); }
        material(...a: any[]) { return rt.wired_scene_types.node_material(this, ...a); }
        mesh(...a: any[]) { return rt.wired_scene_types.node_mesh(this, ...a); }
        name(...a: any[]) { return rt.wired_scene_types.node_name(this, ...a); }
        parent(...a: any[]) { return rt.wired_scene_types.node_parent(this, ...a); }
        removeChild(...a: any[]) { return rt.wired_scene_types.node_remove_child(this, ...a); }
        rigidBody(...a: any[]) { return rt.wired_scene_types.node_rigid_body(this, ...a); }
        rotation(...a: any[]) { return rt.wired_scene_types.node_rotation(this, ...a); }
        scale(...a: any[]) { return rt.wired_scene_types.node_scale(this, ...a); }
        setCollider(...a: any[]) { return rt.wired_scene_types.node_set_collider(this, ...a); }
        setMaterial(...a: any[]) { return rt.wired_scene_types.node_set_material(this, ...a); }
        setMesh(...a: any[]) { return rt.wired_scene_types.node_set_mesh(this, ...a); }
        setName(...a: any[]) { return rt.wired_scene_types.node_set_name(this, ...a); }
        setRigidBody(...a: any[]) { return rt.wired_scene_types.node_set_rigid_body(this, ...a); }
        setRotation(...a: any[]) { return rt.wired_scene_types.node_set_rotation(this, ...a); }
        setScale(...a: any[]) { return rt.wired_scene_types.node_set_scale(this, ...a); }
        setSync(...a: any[]) { return rt.wired_scene_types.node_set_sync(this, ...a); }
        setTransform(...a: any[]) { return rt.wired_scene_types.node_set_transform(this, ...a); }
        setTranslation(...a: any[]) { return rt.wired_scene_types.node_set_translation(this, ...a); }
        sync(...a: any[]) { return rt.wired_scene_types.node_sync(this, ...a); }
        transform(...a: any[]) { return rt.wired_scene_types.node_transform(this, ...a); }
        translation(...a: any[]) { return rt.wired_scene_types.node_translation(this, ...a); }
      },
    },
    "wired:wds/api": {
      getWds: rt.wired_wds.get_wds,
    },
    "wired:wds/types": {
      QueryFuture: class {
        poll(...a: any[]) { return rt.wired_wds_types.query_future_poll(this, ...a); }
      },
      ReadFuture: class {
        poll(...a: any[]) { return rt.wired_wds_types.read_future_poll(this, ...a); }
      },
      Wds: class {
        query(...a: any[]) { return rt.wired_wds_types.wds_query(this, ...a); }
        read(...a: any[]) { return rt.wired_wds_types.wds_read(this, ...a); }
      },
    },
  };
}
