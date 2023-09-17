import { Resource, Warehouse } from "houseki/core";
import { Geometry, Mesh, Name, StandardMaterial } from "houseki/scene";
import { Commands, Entity, Mut, Query, Res, With } from "thyseus";

import { EditorId } from "../../client/components";
import { syncedStore } from "../store";

export function createMeshes(
  commands: Commands,
  warehouse: Res<Mut<Warehouse>>,
  primitives: Query<[Entity, EditorId, Mut<Name>, Mut<Geometry>], With<Mesh>>
) {
  const ids: string[] = [];

  for (const [ent, editorId, name, geometry] of primitives) {
    ids.push(editorId.value);

    const mesh = syncedStore.meshes[editorId.value];

    if (!mesh) {
      // Mesh is gone, remove it from the scene
      commands.get(ent).despawn();
    } else {
      // Sync mesh
      name.value = mesh.name;

      setAttribute(mesh.indices, geometry.indices, warehouse, Uint32Array);
      setAttribute(mesh.positions, geometry.positions, warehouse);
      setAttribute(mesh.normals, geometry.normals, warehouse);
      setAttribute(mesh.uv, geometry.uv, warehouse);
      setAttribute(mesh.uv1, geometry.uv1, warehouse);
      setAttribute(mesh.uv2, geometry.uv2, warehouse);
      setAttribute(mesh.uv3, geometry.uv3, warehouse);
      setAttribute(mesh.colors, geometry.colors, warehouse);
      setAttribute(mesh.joints, geometry.joints, warehouse);
      setAttribute(mesh.weights, geometry.weights, warehouse);
    }
  }

  // Create new meshes
  for (const id of Object.keys(syncedStore.meshes)) {
    if (!ids.includes(id)) {
      commands
        .spawn(true)
        .add(new EditorId(id))
        .addType(Name)
        .addType(Mesh)
        .addType(Geometry)
        .addType(StandardMaterial);
    }
  }
}

type TypedArrayConstructor = Float32ArrayConstructor | Uint32ArrayConstructor;
type TypedArray = Float32Array | Uint32Array;

function setAttribute<T extends TypedArray = Float32Array>(
  data: number[],
  resource: Resource<T>,
  warehouse: Warehouse,
  Constructor: TypedArrayConstructor = Float32Array
) {
  const array = resource.read(warehouse);

  if (array) {
    // Assume if same length, same data
    if (array.length === data.length) {
      return;
    }

    array.set(data);
  } else {
    const newArray = new Constructor(data) as T;
    resource.write(newArray, warehouse);
  }
}
