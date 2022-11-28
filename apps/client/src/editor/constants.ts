import { AutoCollider, BoxMesh, Material, Node, SceneJSON } from "engine";

import { hexToRgb } from "../utils/rgb";

const groundMaterial = new Material();
groundMaterial.name = "Ground";

const color = "#28E635";
const rgb = hexToRgb(color);
groundMaterial.color = [rgb[0] / 255, rgb[1] / 255, rgb[2] / 255, 255];

const groundMesh = new BoxMesh();
groundMesh.name = "Ground";
groundMesh.width = 10;
groundMesh.height = 0.2;
groundMesh.depth = 10;
groundMesh.materialId = groundMaterial.id;

const groundNode = new Node();
groundNode.name = "Ground";
groundNode.meshId = groundMesh.id;
groundNode.position = [0, -1, 0];
groundNode.collider = new AutoCollider();

const spawnNode = new Node();
spawnNode.name = "Spawn";

export const DEFAULT_SCENE: SceneJSON = {
  spawnId: spawnNode.id,
  nodes: [spawnNode.toJSON(), groundNode.toJSON()],
  meshes: [groundMesh.toJSON()],
  materials: [groundMaterial.toJSON()],
  accessors: [],
  images: [],
  animations: [],
};
