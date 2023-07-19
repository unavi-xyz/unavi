import { Warehouse } from "lattice-engine/core";
import { EventWriter, Res } from "thyseus";

import { useClientStore } from "../clientStore";
import { AddMesh, AddNode, EditMesh, EditNode } from "../editor/events";
import { PlayerJoin, PlayerLeave } from "../events";

export function sendEvents(
  warehouse: Res<Warehouse>,
  playerJoin: EventWriter<PlayerJoin>,
  playerLeave: EventWriter<PlayerLeave>,
  addNode: EventWriter<AddNode>,
  addMesh: EventWriter<AddMesh>,
  editNode: EventWriter<EditNode>,
  editMesh: EventWriter<EditMesh>
) {
  const events = useClientStore.getState().events;

  // Only send 1 event per frame
  const event = events.shift();
  if (!event) return;

  switch (event.id) {
    case "com.wired-protocol.world.player.join": {
      const e = playerJoin.create();
      e.playerId = event.data.playerId;
      break;
    }

    case "com.wired-protocol.world.player.leave": {
      const e = playerLeave.create();
      e.playerId = event.data;
      break;
    }

    case "xyz.unavi.editor.add.node": {
      const e = addNode.create();
      e.name = event.data.name;
      break;
    }

    case "xyz.unavi.editor.add.mesh": {
      const e = addMesh.create();
      e.name = event.data.name;
      break;
    }

    case "xyz.unavi.editor.edit.node": {
      const e = editNode.create();
      e.target = event.data.target;

      e.name = event.data.name ?? "";
      e.meshName = event.data.mesh ?? "";
      e.parentName = event.data.parent ?? "";
      break;
    }

    case "xyz.unavi.editor.edit.mesh": {
      const e = editMesh.create();
      e.target = event.data.target;

      e.name = event.data.name ?? "";
      e.materialName = event.data.material ?? "";

      const indices = new Uint32Array(event.data.indices?.length ?? 0);
      indices.set(event.data.indices ?? []);
      e.indices.write(indices, warehouse);

      const colors = new Float32Array(event.data.colors?.length ?? 0);
      colors.set(event.data.colors ?? []);
      e.colors.write(colors, warehouse);

      const joints = new Float32Array(event.data.joints?.length ?? 0);
      joints.set(event.data.joints ?? []);
      e.joints.write(joints, warehouse);

      const normals = new Float32Array(event.data.normals?.length ?? 0);
      normals.set(event.data.normals ?? []);
      e.normals.write(normals, warehouse);

      const positions = new Float32Array(event.data.positions?.length ?? 0);
      positions.set(event.data.positions ?? []);
      e.positions.write(positions, warehouse);

      const uv = new Float32Array(event.data.uv?.length ?? 0);
      uv.set(event.data.uv ?? []);
      e.uv.write(uv, warehouse);

      const uv1 = new Float32Array(event.data.uv1?.length ?? 0);
      uv1.set(event.data.uv1 ?? []);
      e.uv1.write(uv1, warehouse);

      const uv2 = new Float32Array(event.data.uv2?.length ?? 0);
      uv2.set(event.data.uv2 ?? []);
      e.uv2.write(uv2, warehouse);

      const uv3 = new Float32Array(event.data.uv3?.length ?? 0);
      uv3.set(event.data.uv3 ?? []);
      e.uv3.write(uv3, warehouse);

      const weights = new Float32Array(event.data.weights?.length ?? 0);
      weights.set(event.data.weights ?? []);
      e.weights.write(weights, warehouse);
      break;
    }
  }
}
