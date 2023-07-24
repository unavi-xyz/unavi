import { Warehouse } from "lattice-engine/core";
import { Geometry, Mesh, Name } from "lattice-engine/scene";
import { Entity, EventReader, Mut, Query, Res } from "thyseus";

import { EditMesh } from "../events";

export function editMeshes(
  warehouse: Res<Warehouse>,
  events: EventReader<EditMesh>,
  meshes: Query<[Mut<Mesh>, Mut<Name>, Mut<Geometry>]>,
  named: Query<[Entity, Name]>
) {
  if (events.length === 0) return;

  for (const e of events) {
    for (const [mesh, name, geometry] of meshes) {
      if (name.value !== e.target) continue;

      if (e.name) {
        name.value = e.name;
      }

      if (e.indices) {
        geometry.indices.write(e.indices.read(warehouse), warehouse);
      }

      if (e.colors) {
        geometry.colors.write(e.colors.read(warehouse), warehouse);
      }

      if (e.joints) {
        geometry.joints.write(e.joints.read(warehouse), warehouse);
      }

      if (e.normals) {
        geometry.normals.write(e.normals.read(warehouse), warehouse);
      }

      if (e.positions) {
        geometry.positions.write(e.positions.read(warehouse), warehouse);
      }

      if (e.uv) {
        geometry.uv.write(e.uv.read(warehouse), warehouse);
      }

      if (e.uv1) {
        geometry.uv1.write(e.uv1.read(warehouse), warehouse);
      }

      if (e.uv2) {
        geometry.uv2.write(e.uv2.read(warehouse), warehouse);
      }

      if (e.uv3) {
        geometry.uv3.write(e.uv3.read(warehouse), warehouse);
      }

      if (e.weights) {
        geometry.weights.write(e.weights.read(warehouse), warehouse);
      }

      if (e.materialName) {
        for (const [entity, name] of named) {
          if (name.value === e.materialName) {
            mesh.materialId = entity.id;
            break;
          }
        }
      }
    }
  }

  events.clear();
}
