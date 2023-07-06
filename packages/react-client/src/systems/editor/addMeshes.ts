import { Warehouse } from "lattice-engine/core";
import { Geometry, Mesh, Name } from "lattice-engine/scene";
import { Commands, dropStruct, Entity, EventReader, Query, Res } from "thyseus";

import { AddMesh } from "../../events";

export function addMeshes(
  commands: Commands,
  warehouse: Res<Warehouse>,
  addMesh: EventReader<AddMesh>,
  named: Query<[Entity, Name]>
) {
  if (addMesh.length === 0) return;

  const nameComp = new Name();

  for (const event of addMesh) {
    nameComp.value = event.name;

    const geometry = new Geometry();

    geometry.indices.write(event.indices.read(warehouse), warehouse);
    geometry.colors.write(event.colors.read(warehouse), warehouse);
    geometry.joints.write(event.joints.read(warehouse), warehouse);
    geometry.normals.write(event.normals.read(warehouse), warehouse);
    geometry.positions.write(event.positions.read(warehouse), warehouse);
    geometry.uv.write(event.uv.read(warehouse), warehouse);
    geometry.uv1.write(event.uv1.read(warehouse), warehouse);
    geometry.uv2.write(event.uv2.read(warehouse), warehouse);
    geometry.uv3.write(event.uv3.read(warehouse), warehouse);
    geometry.weights.write(event.weights.read(warehouse), warehouse);

    const meshComp = new Mesh();

    for (const [entity, name] of named) {
      if (name.value === event.materialName) {
        meshComp.materialId = entity.id;
        break;
      }
    }

    commands.spawn(true).add(nameComp).add(meshComp).add(geometry).id;

    dropStruct(geometry);
    dropStruct(meshComp);
  }

  dropStruct(nameComp);

  addMesh.clear();
}
