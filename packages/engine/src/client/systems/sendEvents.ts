import { EditorEvent } from "@unavi/protocol";
import { Warehouse } from "lattice-engine/core";
import { EventWriter, Res } from "thyseus";

import {
  AddMesh,
  AddNode,
  EditCollider,
  EditExtra,
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
  warehouse: Res<Warehouse>,
  playerJoin: EventWriter<PlayerJoin>,
  playerLeave: EventWriter<PlayerLeave>,
  addNode: EventWriter<AddNode>,
  addMesh: EventWriter<AddMesh>,
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
      const e = playerJoin.create();
      e.playerId = msg.response.playerJoined.playerId;
      break;
    }

    case "playerLeft": {
      const e = playerLeave.create();
      e.playerId = msg.response.playerLeft.playerId;
      break;
    }

    case "event": {
      const editor = EditorEvent.fromBinary(msg.response.event.data);

      switch (editor.event.oneofKind) {
        case "addNode": {
          const e = addNode.create();
          e.name = editor.event.addNode.id;
          break;
        }

        case "addMesh": {
          const e = addMesh.create();
          e.name = editor.event.addMesh.id;
          break;
        }

        case "editNode": {
          const e = editNode.create();
          e.target = editor.event.editNode.target;

          e.name = editor.event.editNode.name ?? "";
          e.parentName = editor.event.editNode.parent ?? "";
          e.meshName = editor.event.editNode.mesh ?? "";

          if (editor.event.editNode.translation) {
            e.translation = true;
            e.transform.translation.set(
              editor.event.editNode.translation[0] ?? 0,
              editor.event.editNode.translation[1] ?? 0,
              editor.event.editNode.translation[2] ?? 0
            );
          }

          if (editor.event.editNode.rotation) {
            e.rotation = true;
            e.transform.rotation.set(
              editor.event.editNode.rotation[0] ?? 0,
              editor.event.editNode.rotation[1] ?? 0,
              editor.event.editNode.rotation[2] ?? 0,
              editor.event.editNode.rotation[3] ?? 0
            );
          }

          if (editor.event.editNode.scale) {
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
              const extra = editExta.create();
              extra.target = editor.event.editNode.target;
              extra.key = key;
              extra.value = JSON.stringify(value);
            }
          }

          if (editor.event.editNode.rigidBody) {
            const rigidBodyEvent = rigidBody.create();
            rigidBodyEvent.target = editor.event.editNode.target;
            rigidBodyEvent.type = editor.event.editNode.rigidBody.type;
          }

          if (editor.event.editNode.collider) {
            const colliderEvent = collider.create();
            colliderEvent.target = editor.event.editNode.target;

            colliderEvent.type = editor.event.editNode.collider.type;

            if (editor.event.editNode.collider.size) {
              colliderEvent.size.set(editor.event.editNode.collider.size);
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
          }

          break;
        }

        case "editMesh": {
          const e = editMesh.create();
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
        }
      }
    }
  }
}
