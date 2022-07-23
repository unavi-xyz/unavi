import { AnimationAction, AnimationClip, AnimationMixer, Group, Object3D } from "three";

import { GLTF } from "../schemaTypes";

export type SceneResult = {
  scene: Group;
  animationActions: AnimationAction[];
  animationMixer: AnimationMixer;
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

  // Load Nodes
  const startNodes = performance.now();

  const nodePromises = sceneDef.nodes?.map((nodeIndex) => buildNodeHierarchy(nodeIndex)) ?? [];
  const nodes = await Promise.all(nodePromises);
  scene.add(...nodes);

  const endNodes = performance.now();
  console.log(`Loaded ${nodePromises.length} nodes in ${(endNodes - startNodes) / 1000} seconds`);

  // Load Animations
  const animationPromises = json.animations?.map((_, index) => loadAnimation(index));
  const animationClips = await Promise.all(animationPromises ?? []);

  const animationMixer = new AnimationMixer(scene);
  const animationActions = animationClips.map((clip) => animationMixer.clipAction(clip));

  const endAnimations = performance.now();
  console.log(
    `Loaded ${animationClips.length} animations in ${(endAnimations - endNodes) / 1000} seconds`
  );

  return {
    scene,
    animationActions,
    animationMixer,
  };
}
