import { Warehouse } from "houseki/core";
import { GltfInfo, SceneView, SubScene } from "houseki/gltf";
import {
  BoxCollider,
  CapsuleCollider,
  CylinderCollider,
  DynamicBody,
  HullCollider,
  MeshCollider,
  SphereCollider,
  StaticBody,
} from "houseki/physics";
import { Geometry, Mesh, Name, Parent, Transform } from "houseki/scene";
import { Commands, Entity, Query, Res, With } from "thyseus";

import { EditorId, WorldJson } from "../../client/components";
import { addMesh, addNode, editMesh, editNode } from "../actions";
import { addScene } from "../actions/addScene";
import { editScene } from "../actions/editScene";
import { getId, setEntityId } from "../entities";
import {
  SyncedNode_Collider,
  SyncedNode_Collider_Type,
  SyncedNode_RigidBody,
  SyncedNode_RigidBody_Type,
  syncedStore,
} from "../store";

export function initSyncedStore(
  commands: Commands,
  warehouse: Res<Warehouse>,
  worlds: Query<[SceneView, GltfInfo], With<WorldJson>>,
  subscenes: Query<[Entity, Name, SubScene]>,
  nodes: Query<[Entity, Name, Transform, Parent]>,
  primitives: Query<[Entity, Name, Geometry, Mesh]>,
  boxColliders: Query<[Entity, BoxCollider]>,
  sphereColliders: Query<[Entity, SphereCollider]>,
  capsuleColliders: Query<[Entity, CapsuleCollider]>,
  cylinderColliders: Query<[Entity, CylinderCollider]>,
  meshColliders: Query<[Entity, MeshCollider]>,
  hullColliders: Query<[Entity, HullCollider]>,
  staticBodies: Query<Entity, With<StaticBody>>,
  dynamicBodies: Query<Entity, With<DynamicBody>>
) {
  if (syncedStore.initialized) return;

  for (const [view, info] of worlds) {
    // Nodes
    for (const entityId of info.nodes) {
      for (const [ent, name, transform] of nodes) {
        if (ent.id !== entityId) continue;

        const id = addNode(name.value);
        setEntityId(id, ent.id);

        commands.getById(entityId).add(new EditorId(id));

        const collider: SyncedNode_Collider = {
          height: 1,
          meshId: "",
          radius: 0.5,
          size: [1, 1, 1],
          type: SyncedNode_Collider_Type.NONE,
        };

        for (const [boxEnt, col] of boxColliders) {
          if (boxEnt.id !== entityId) continue;
          collider.size = col.size.toArray();
          collider.type = SyncedNode_Collider_Type.BOX;
        }

        for (const [sphereEnt, col] of sphereColliders) {
          if (sphereEnt.id !== entityId) continue;
          collider.radius = col.radius;
          collider.type = SyncedNode_Collider_Type.SPHERE;
        }

        for (const [capsuleEnt, col] of capsuleColliders) {
          if (capsuleEnt.id !== entityId) continue;
          collider.height = col.height;
          collider.radius = col.radius;
          collider.type = SyncedNode_Collider_Type.CAPSULE;
        }

        for (const [cylinderEnt, col] of cylinderColliders) {
          if (cylinderEnt.id !== entityId) continue;
          collider.height = col.height;
          collider.radius = col.radius;
          collider.type = SyncedNode_Collider_Type.CYLINDER;
        }

        for (const [meshEnt, col] of meshColliders) {
          if (meshEnt.id !== entityId) continue;
          const meshId = getId(col.meshId);
          if (!meshId) continue;
          collider.meshId = meshId;
          collider.type = SyncedNode_Collider_Type.MESH;
        }

        for (const [hullEnt, col] of hullColliders) {
          if (hullEnt.id !== entityId) continue;
          const meshId = getId(col.meshId);
          if (!meshId) continue;
          collider.meshId = meshId;
          collider.type = SyncedNode_Collider_Type.HULL;
        }

        const rigidBody: SyncedNode_RigidBody = {
          type: SyncedNode_RigidBody_Type.STATIC,
        };

        for (const staticEnt of staticBodies) {
          if (staticEnt.id !== entityId) continue;
          rigidBody.type = SyncedNode_RigidBody_Type.STATIC;
        }

        for (const dynamicEnt of dynamicBodies) {
          if (dynamicEnt.id !== entityId) continue;
          rigidBody.type = SyncedNode_RigidBody_Type.DYNAMIC;
        }

        editNode(id, {
          collider,
          rigidBody,
          rotation: transform.rotation.toArray(),
          scale: transform.scale.toArray(),
          translation: transform.translation.toArray(),
        });
      }
    }

    // Scenes
    for (const entityId of view.scenes) {
      for (const [ent, name, subscene] of subscenes) {
        if (ent.id !== entityId) continue;

        const id = addScene(name.value);
        setEntityId(id, ent.id);

        commands.getById(entityId).add(new EditorId(id));

        if (view.active === entityId) {
          syncedStore.defaultSceneId = id;
        }

        const nodeIds = [];

        for (const nodeId of subscene.nodes) {
          const id = getId(nodeId);
          if (!id) continue;

          nodeIds.push(id);
        }

        editScene(id, {
          nodeIds,
        });
      }
    }

    // Node parenting
    for (const entityId of info.nodes) {
      const id = getId(entityId);
      if (!id) continue;

      for (const [ent, , , parent] of nodes) {
        if (ent.id !== entityId) continue;

        const parentId = getId(parent.id);
        if (!parentId) continue;

        let isScene = false;

        for (const sceneId of view.scenes) {
          if (sceneId === parent.id) {
            isScene = true;
            break;
          }
        }

        if (!isScene) {
          editNode(id, { parentId });
        }
      }
    }

    // Meshes
    for (const entityId of info.meshPrimitives) {
      for (const [ent, name, geometry, mesh] of primitives) {
        if (ent.id !== entityId) continue;

        const id = addMesh(name.value);
        setEntityId(id, ent.id);

        commands.getById(entityId).add(new EditorId(id));

        editMesh(id, {
          colors: Array.from(geometry.colors.read(warehouse) ?? []),
          indices: Array.from(geometry.indices.read(warehouse) ?? []),
          joints: Array.from(geometry.joints.read(warehouse) ?? []),
          normals: Array.from(geometry.normals.read(warehouse) ?? []),
          positions: Array.from(geometry.positions.read(warehouse) ?? []),
          uv: Array.from(geometry.uv.read(warehouse) ?? []),
          uv1: Array.from(geometry.uv1.read(warehouse) ?? []),
          uv2: Array.from(geometry.uv2.read(warehouse) ?? []),
          uv3: Array.from(geometry.uv3.read(warehouse) ?? []),
          weights: Array.from(geometry.weights.read(warehouse) ?? []),
        });

        const nodeId = getId(mesh.parentId);

        if (nodeId) {
          editNode(nodeId, {
            meshId: id,
          });
        }
      }
    }
  }

  syncedStore.initialized = true;
}
