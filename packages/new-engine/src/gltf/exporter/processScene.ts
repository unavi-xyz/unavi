import { Object3D, Scene } from "three";

import { GLTF, Scene as SceneDef } from "../schemaTypes";

export function processScene(scene: Scene, json: GLTF, processNode: (node: Object3D) => number) {
  const sceneDef: SceneDef = {};
  if (scene.name) sceneDef.name = scene.name;

  // Add to json
  if (json.scenes === undefined) json.scenes = [];
  json.scenes.push(sceneDef);

  // Load children
  const nodes: number[] = [];

  scene.children.forEach((child) => {
    if (!child.visible) return;
    const nodeIndex = processNode(child);
    nodes.push(nodeIndex);
  });

  if (nodes.length > 0) sceneDef.nodes = nodes;
}
