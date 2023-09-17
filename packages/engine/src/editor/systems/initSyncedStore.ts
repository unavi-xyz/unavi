import { Warehouse } from "houseki/core";
import { GltfInfo, SceneView, SubScene } from "houseki/gltf";
import { Geometry, Mesh, Name, Parent, Transform } from "houseki/scene";
import { Commands, Entity, Query, Res, With } from "thyseus";

import { EditorId, WorldJson } from "../../client/components";
import { addMesh, addNode, editMesh, editNode } from "../actions";
import { addScene } from "../actions/addScene";
import { editScene } from "../actions/editScene";
import { getId, setEntityId } from "../entities";
import { syncedStore } from "../store";

export function initSyncedStore(
  commands: Commands,
  warehouse: Res<Warehouse>,
  worlds: Query<[SceneView, GltfInfo], With<WorldJson>>,
  subscenes: Query<[Entity, Name, SubScene]>,
  nodes: Query<[Entity, Name, Transform, Parent]>,
  primitives: Query<[Entity, Name, Geometry, Mesh]>
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

        editNode(id, {
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
