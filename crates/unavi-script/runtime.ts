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
        bone() {
          return rt.wired_agent_types.agent_bone();
        }
      },
    },
    "wired:event/api": {
      emit: rt.wired_event.emit,
      listen: rt.wired_event.listen,
    },
    "wired:event/types": {
      EventReceptor: class {
        poll() {
          return rt.wired_event_types.event_receptor_poll();
        }
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
        poll() {
          return rt.wired_input_types.input_listener_poll();
        }
      },
    },
    "wired:portal/api": {
      listPortals: rt.wired_portal.list_portals,
      openPortal: rt.wired_portal.open_portal,
    },
    "wired:portal/types": {
      Portal: class {
        close() {
          return rt.wired_portal_types.portal_close();
        }
        destination() {
          return rt.wired_portal_types.portal_destination();
        }
        id() {
          return rt.wired_portal_types.portal_id();
        }
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
        addAsset() {
          return rt.wired_scene_types.document_add_asset();
        }
        assets() {
          return rt.wired_scene_types.document_assets();
        }
        clone() {
          return rt.wired_scene_types.document_clone();
        }
        createMaterial() {
          return rt.wired_scene_types.document_create_material();
        }
        createMesh() {
          return rt.wired_scene_types.document_create_mesh();
        }
        createNode() {
          return rt.wired_scene_types.document_create_node();
        }
        globalTransform() {
          return rt.wired_scene_types.document_global_transform();
        }
        id() {
          return rt.wired_scene_types.document_id();
        }
        materials() {
          return rt.wired_scene_types.document_materials();
        }
        meshes() {
          return rt.wired_scene_types.document_meshes();
        }
        nodes() {
          return rt.wired_scene_types.document_nodes();
        }
        public() {
          return rt.wired_scene_types.document_public();
        }
        removeAsset() {
          return rt.wired_scene_types.document_remove_asset();
        }
        removeMaterial() {
          return rt.wired_scene_types.document_remove_material();
        }
        removeMesh() {
          return rt.wired_scene_types.document_remove_mesh();
        }
        removeNode() {
          return rt.wired_scene_types.document_remove_node();
        }
        roots() {
          return rt.wired_scene_types.document_roots();
        }
        rotation() {
          return rt.wired_scene_types.document_rotation();
        }
        scale() {
          return rt.wired_scene_types.document_scale();
        }
        setPublic() {
          return rt.wired_scene_types.document_set_public();
        }
        setRotation() {
          return rt.wired_scene_types.document_set_rotation();
        }
        setScale() {
          return rt.wired_scene_types.document_set_scale();
        }
        setSync() {
          return rt.wired_scene_types.document_set_sync();
        }
        setTransform() {
          return rt.wired_scene_types.document_set_transform();
        }
        setTranslation() {
          return rt.wired_scene_types.document_set_translation();
        }
        sync() {
          return rt.wired_scene_types.document_sync();
        }
        transform() {
          return rt.wired_scene_types.document_transform();
        }
        translation() {
          return rt.wired_scene_types.document_translation();
        }
      },
      Material: class {
        alphaCutoff() {
          return rt.wired_scene_types.material_alpha_cutoff();
        }
        alphaMode() {
          return rt.wired_scene_types.material_alpha_mode();
        }
        baseColor() {
          return rt.wired_scene_types.material_base_color();
        }
        clone() {
          return rt.wired_scene_types.material_clone();
        }
        doubleSided() {
          return rt.wired_scene_types.material_double_sided();
        }
        id() {
          return rt.wired_scene_types.material_id();
        }
        metallic() {
          return rt.wired_scene_types.material_metallic();
        }
        name() {
          return rt.wired_scene_types.material_name();
        }
        roughness() {
          return rt.wired_scene_types.material_roughness();
        }
        setAlphaCutoff() {
          return rt.wired_scene_types.material_set_alpha_cutoff();
        }
        setAlphaMode() {
          return rt.wired_scene_types.material_set_alpha_mode();
        }
        setBaseColor() {
          return rt.wired_scene_types.material_set_base_color();
        }
        setDoubleSided() {
          return rt.wired_scene_types.material_set_double_sided();
        }
        setMetallic() {
          return rt.wired_scene_types.material_set_metallic();
        }
        setName() {
          return rt.wired_scene_types.material_set_name();
        }
        setRoughness() {
          return rt.wired_scene_types.material_set_roughness();
        }
        setSync() {
          return rt.wired_scene_types.material_set_sync();
        }
        setUnlit() {
          return rt.wired_scene_types.material_set_unlit();
        }
        sync() {
          return rt.wired_scene_types.material_sync();
        }
        unlit() {
          return rt.wired_scene_types.material_unlit();
        }
      },
      Mesh: class {
        clone() {
          return rt.wired_scene_types.mesh_clone();
        }
        colors() {
          return rt.wired_scene_types.mesh_colors();
        }
        id() {
          return rt.wired_scene_types.mesh_id();
        }
        indices() {
          return rt.wired_scene_types.mesh_indices();
        }
        name() {
          return rt.wired_scene_types.mesh_name();
        }
        normals() {
          return rt.wired_scene_types.mesh_normals();
        }
        positions() {
          return rt.wired_scene_types.mesh_positions();
        }
        setColors() {
          return rt.wired_scene_types.mesh_set_colors();
        }
        setIndices() {
          return rt.wired_scene_types.mesh_set_indices();
        }
        setName() {
          return rt.wired_scene_types.mesh_set_name();
        }
        setNormals() {
          return rt.wired_scene_types.mesh_set_normals();
        }
        setPositions() {
          return rt.wired_scene_types.mesh_set_positions();
        }
        setSync() {
          return rt.wired_scene_types.mesh_set_sync();
        }
        setTangents() {
          return rt.wired_scene_types.mesh_set_tangents();
        }
        setTopology() {
          return rt.wired_scene_types.mesh_set_topology();
        }
        setUv0() {
          return rt.wired_scene_types.mesh_set_uv0();
        }
        setUv1() {
          return rt.wired_scene_types.mesh_set_uv1();
        }
        sync() {
          return rt.wired_scene_types.mesh_sync();
        }
        tangents() {
          return rt.wired_scene_types.mesh_tangents();
        }
        topology() {
          return rt.wired_scene_types.mesh_topology();
        }
        uv0() {
          return rt.wired_scene_types.mesh_uv0();
        }
        uv1() {
          return rt.wired_scene_types.mesh_uv1();
        }
      },
      Node: class {
        addChild() {
          return rt.wired_scene_types.node_add_child();
        }
        children() {
          return rt.wired_scene_types.node_children();
        }
        clone() {
          return rt.wired_scene_types.node_clone();
        }
        collider() {
          return rt.wired_scene_types.node_collider();
        }
        globalTransform() {
          return rt.wired_scene_types.node_global_transform();
        }
        id() {
          return rt.wired_scene_types.node_id();
        }
        material() {
          return rt.wired_scene_types.node_material();
        }
        mesh() {
          return rt.wired_scene_types.node_mesh();
        }
        name() {
          return rt.wired_scene_types.node_name();
        }
        parent() {
          return rt.wired_scene_types.node_parent();
        }
        removeChild() {
          return rt.wired_scene_types.node_remove_child();
        }
        rigidBody() {
          return rt.wired_scene_types.node_rigid_body();
        }
        rotation() {
          return rt.wired_scene_types.node_rotation();
        }
        scale() {
          return rt.wired_scene_types.node_scale();
        }
        setCollider() {
          return rt.wired_scene_types.node_set_collider();
        }
        setMaterial() {
          return rt.wired_scene_types.node_set_material();
        }
        setMesh() {
          return rt.wired_scene_types.node_set_mesh();
        }
        setName() {
          return rt.wired_scene_types.node_set_name();
        }
        setRigidBody() {
          return rt.wired_scene_types.node_set_rigid_body();
        }
        setRotation() {
          return rt.wired_scene_types.node_set_rotation();
        }
        setScale() {
          return rt.wired_scene_types.node_set_scale();
        }
        setSync() {
          return rt.wired_scene_types.node_set_sync();
        }
        setTransform() {
          return rt.wired_scene_types.node_set_transform();
        }
        setTranslation() {
          return rt.wired_scene_types.node_set_translation();
        }
        sync() {
          return rt.wired_scene_types.node_sync();
        }
        transform() {
          return rt.wired_scene_types.node_transform();
        }
        translation() {
          return rt.wired_scene_types.node_translation();
        }
      },
    },
    "wired:wds/api": {
      getWds: rt.wired_wds.get_wds,
    },
    "wired:wds/types": {
      QueryFuture: class {
        poll() {
          return rt.wired_wds_types.query_future_poll();
        }
      },
      ReadFuture: class {
        poll() {
          return rt.wired_wds_types.read_future_poll();
        }
      },
      Wds: class {
        query() {
          return rt.wired_wds_types.wds_query();
        }
        read() {
          return rt.wired_wds_types.wds_read();
        }
      },
    },
  };
}
