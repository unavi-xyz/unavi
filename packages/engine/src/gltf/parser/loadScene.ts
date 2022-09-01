import { AnimationClip, Group, Object3D } from "three";

import { GLTF } from "../schemaTypes";

export type SceneResult = {
  scene: Group;
  animations: AnimationClip[];
};

export async function loadScene(
  index: number,
  json: GLTF,
  boneIndexes: Set<number>,
  skinnedMeshIndexes: Set<number>,
  meshReferenceCount: Map<number, number>,
  buildNodeHierarchy: (index: number) => Promise<Object3D>,
  loadAnimation: (index: number) => Promise<AnimationClip>
): Promise<SceneResult> {
  if (!json.scenes) {
    throw new Error("No scenes found");
  }

  const sceneDef = json.scenes[index];
  const scene = new Group();
  scene.name = sceneDef.name ?? `scene_${index}`;

  // Mark bones
  if (json.skins) {
    json.skins.forEach((skin) => {
      skin.joints.forEach((jointIndex) => {
        boneIndexes.add(jointIndex);
      });
    });
  }

  // Mark meshes
  if (json.nodes) {
    json.nodes.forEach((nodeDef) => {
      if (nodeDef.mesh !== undefined) {
        // Count how many times this mesh is used
        const current = meshReferenceCount.get(nodeDef.mesh) ?? 0;
        meshReferenceCount.set(nodeDef.mesh, current + 1);

        if (nodeDef.skin !== undefined) {
          // Mark that this mesh is a skinned mesh
          skinnedMeshIndexes.add(nodeDef.mesh);
        }
      }
    });
  }

  // Load nodes + animations
  const nodePromises =
    sceneDef.nodes?.map((nodeIndex) => buildNodeHierarchy(nodeIndex)) ?? [];
  const animationPromises =
    json.animations?.map((_, index) => loadAnimation(index)) ?? [];

  const [nodes, animations] = await Promise.all([
    Promise.all(nodePromises),
    Promise.all(animationPromises),
  ]);

  // Add nodes to scene
  scene.add(...nodes);

  return {
    scene,
    animations,
  };
}
