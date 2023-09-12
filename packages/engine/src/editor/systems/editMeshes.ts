import { Warehouse } from "houseki/core";
import { Geometry, Mesh, Name } from "houseki/scene";
import { Entity, EventReader, Mut, Query, Res } from "thyseus";

import { EditorId } from "../../client/components";
import { EditMesh } from "../events";

export function editMeshes(
  warehouse: Res<Mut<Warehouse>>,
  events: EventReader<EditMesh>,
  meshes: Query<[EditorId, Mut<Mesh>, Mut<Name>, Mut<Geometry>]>,
  ids: Query<[Entity, EditorId]>
) {
  if (events.length === 0) return;

  for (const e of events) {
    for (const [id, mesh, name, geometry] of meshes) {
      if (id.value !== e.target) continue;

      if (e.name) {
        name.value = e.name;
      }

      const indices = e.indices.read(warehouse);
      if (indices) {
        geometry.indices.write(indices, warehouse);
      }

      const colors = e.colors.read(warehouse);
      if (colors) {
        geometry.colors.write(colors, warehouse);
      }

      const normals = e.normals.read(warehouse);
      if (normals) {
        geometry.normals.write(normals, warehouse);
      }

      const positions = e.positions.read(warehouse);
      if (positions) {
        geometry.positions.write(positions, warehouse);
      }

      const weights = e.weights.read(warehouse);
      if (weights) {
        geometry.weights.write(weights, warehouse);
      }

      const joints = e.joints.read(warehouse);
      if (joints) {
        geometry.joints.write(joints, warehouse);
      }

      const uv = e.uv.read(warehouse);
      if (uv) {
        geometry.uv.write(uv, warehouse);
      }

      const uv1 = e.uv1.read(warehouse);
      if (uv1) {
        geometry.uv1.write(uv1, warehouse);
      }

      const uv2 = e.uv2.read(warehouse);
      if (uv2) {
        geometry.uv2.write(uv2, warehouse);
      }

      const uv3 = e.uv3.read(warehouse);
      if (uv3) {
        geometry.uv3.write(uv3, warehouse);
      }

      if (e.material) {
        for (const [entity, materialId] of ids) {
          if (materialId.value === e.material) {
            mesh.materialId = entity.id;
            break;
          }
        }
      }
    }
  }

  events.clear();
}
