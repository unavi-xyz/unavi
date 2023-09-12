import { EditorEvent } from "@unavi/protocol";
import { Warehouse } from "houseki/core";
import { EventWriter, Mut, Res } from "thyseus";

import {
  AddMesh,
  AddNode,
  EditCollider,
  EditExtra,
  EditId,
  EditMesh,
  EditNode,
  EditRigidBody,
} from "../../editor/events";
import { useClientStore } from "../clientStore";
import { PlayerJoin, PlayerLeave } from "../events";

/**
 * Converts networked events into ECS events
 */
export function sendEvents(
  warehouse: Res<Mut<Warehouse>>,
  playerJoin: EventWriter<PlayerJoin>,
  playerLeave: EventWriter<PlayerLeave>,
  addNode: EventWriter<AddNode>,
  addMesh: EventWriter<AddMesh>,
  editId: EventWriter<EditId>,
  editNode: EventWriter<EditNode>,
  editMesh: EventWriter<EditMesh>,
  editExta: EventWriter<EditExtra>,
  rigidBody: EventWriter<EditRigidBody>,
  collider: EventWriter<EditCollider>
) {
  const ecsIncoming = useClientStore.getState().ecsIncoming;

  // Only send 1 event per frame
  const msg = ecsIncoming.shift();
  if (!msg) return;

  switch (msg.response.oneofKind) {
    case "playerJoined": {
      const e = new PlayerJoin();
      e.playerId = msg.response.playerJoined.playerId;
      playerJoin.create(e);
      break;
    }

    case "playerLeft": {
      const e = new PlayerLeave();
      e.playerId = msg.response.playerLeft.playerId;
      playerLeave.create(e);
      break;
    }

    case "event": {
      const editor = EditorEvent.fromBinary(msg.response.event.data);

      console.log("--- event", editor.event.oneofKind);

      switch (editor.event.oneofKind) {
        case "addNode": {
          const e = new AddNode();
          e.id = editor.event.addNode.id;
          addNode.create(e);
          break;
        }

        case "addMesh": {
          const e = new AddMesh();
          e.id = editor.event.addMesh.id;
          addMesh.create(e);
          break;
        }

        case "editId": {
          const e = new EditId();
          e.type = editor.event.editId.type;
          e.index = editor.event.editId.index;
          e.id = editor.event.editId.id;
          editId.create(e);
          break;
        }

        case "editNode": {
          const e = new EditNode();
          e.target = editor.event.editNode.target;

          e.name = editor.event.editNode.name ?? "";
          e.parent = editor.event.editNode.parent ?? "";
          e.mesh = editor.event.editNode.mesh ?? "";

          if (editor.event.editNode.translation.length) {
            e.translation = true;
            e.transform.translation.set(
              editor.event.editNode.translation[0] ?? 0,
              editor.event.editNode.translation[1] ?? 0,
              editor.event.editNode.translation[2] ?? 0
            );
          }

          if (editor.event.editNode.rotation.length) {
            e.rotation = true;
            e.transform.rotation.set(
              editor.event.editNode.rotation[0] ?? 0,
              editor.event.editNode.rotation[1] ?? 0,
              editor.event.editNode.rotation[2] ?? 0,
              editor.event.editNode.rotation[3] ?? 0
            );
          }

          if (editor.event.editNode.scale.length) {
            e.scale = true;
            e.transform.scale.set(
              editor.event.editNode.scale[0] ?? 0,
              editor.event.editNode.scale[1] ?? 0,
              editor.event.editNode.scale[2] ?? 0
            );
          }

          if (editor.event.editNode.extras) {
            for (const [key, value] of Object.entries(
              editor.event.editNode.extras
            )) {
              const extra = new EditExtra();
              extra.target = editor.event.editNode.target;
              extra.key = key;
              extra.value = JSON.stringify(value);
              editExta.create(extra);
            }
          }

          if (editor.event.editNode.rigidBody) {
            const rigidBodyEvent = new EditRigidBody();
            rigidBodyEvent.target = editor.event.editNode.target;
            rigidBodyEvent.type = editor.event.editNode.rigidBody.type;
            rigidBody.create(rigidBodyEvent);
          }

          if (editor.event.editNode.collider) {
            const colliderEvent = new EditCollider();

            colliderEvent.target = editor.event.editNode.target;
            colliderEvent.type = editor.event.editNode.collider.type;

            if (editor.event.editNode.collider.size) {
              colliderEvent.size[0] =
                editor.event.editNode.collider.size[0] ?? 0;
              colliderEvent.size[1] =
                editor.event.editNode.collider.size[1] ?? 0;
              colliderEvent.size[2] =
                editor.event.editNode.collider.size[2] ?? 0;
            }

            if (editor.event.editNode.collider.height) {
              colliderEvent.height = editor.event.editNode.collider.height;
            }

            if (editor.event.editNode.collider.radius) {
              colliderEvent.radius = editor.event.editNode.collider.radius;
            }

            if (editor.event.editNode.collider.mesh) {
              colliderEvent.mesh = editor.event.editNode.collider.mesh;
            }

            collider.create(colliderEvent);
          }

          editNode.create(e);

          break;
        }

        case "editMesh": {
          const e = new EditMesh();
          e.target = editor.event.editMesh.target;

          e.name = editor.event.editMesh.name ?? "";
          e.material = editor.event.editMesh.material ?? "";

          if (editor.event.editMesh.indices) {
            const indices = Uint32Array.from(editor.event.editMesh.indices);
            e.indices.write(indices, warehouse);
          }

          if (editor.event.editMesh.position) {
            const positions = Float32Array.from(editor.event.editMesh.position);
            e.positions.write(positions, warehouse);
          }

          if (editor.event.editMesh.normal) {
            const normals = Float32Array.from(editor.event.editMesh.normal);
            e.normals.write(normals, warehouse);
          }

          if (editor.event.editMesh.color) {
            const colors = Float32Array.from(editor.event.editMesh.color);
            e.colors.write(colors, warehouse);
          }

          if (editor.event.editMesh.weights) {
            const weights = Float32Array.from(editor.event.editMesh.weights);
            e.weights.write(weights, warehouse);
          }

          if (editor.event.editMesh.joints) {
            const joints = Float32Array.from(editor.event.editMesh.joints);
            e.joints.write(joints, warehouse);
          }

          if (editor.event.editMesh.uv) {
            const uvs = Float32Array.from(editor.event.editMesh.uv);
            e.uv.write(uvs, warehouse);
          }

          if (editor.event.editMesh.uv1) {
            const uv1s = Float32Array.from(editor.event.editMesh.uv1);
            e.uv1.write(uv1s, warehouse);
          }

          if (editor.event.editMesh.uv2) {
            const uv2s = Float32Array.from(editor.event.editMesh.uv2);
            e.uv2.write(uv2s, warehouse);
          }

          if (editor.event.editMesh.uv3) {
            const uv3s = Float32Array.from(editor.event.editMesh.uv3);
            e.uv3.write(uv3s, warehouse);
          }

          editMesh.create(e);
        }
      }
    }
  }
}
