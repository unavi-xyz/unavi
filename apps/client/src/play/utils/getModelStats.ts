import { Document, Node } from "@gltf-transform/core";

export interface ModelStats {
  fileSize: number;
  materialCount: number;
  meshCount: number;
  skinCount: number;
  boneCount: number;
  triangleCount: number;
}

export async function getModelStats(doc: Document, array: Uint8Array): Promise<ModelStats> {
  // Get stats
  const materialCount = doc.getRoot().listMaterials().length;
  const meshCount = doc.getRoot().listMeshes().length;
  const skinCount = doc.getRoot().listSkins().length;

  const boneCount = doc
    .getRoot()
    .listSkins()
    .reduce((acc, skin) => {
      skin.listJoints().forEach((n) => acc.add(n));
      return acc;
    }, new Set<Node>()).size;

  const triangleCount = doc
    .getRoot()
    .listMeshes()
    .reduce((acc, mesh) => {
      return (
        acc +
        mesh.listPrimitives().reduce((acc, primitive) => {
          const mode = primitive.getMode();
          if (mode !== 4) return acc;

          const indices = primitive.getIndices();
          if (indices) return acc + indices.getCount() / 3;

          const position = primitive.getAttribute("POSITION");
          if (position) return acc + position.getCount() / 3;

          return acc;
        }, 0)
      );
    }, 0);

  return {
    boneCount,
    fileSize: array.byteLength,
    materialCount,
    meshCount,
    skinCount,
    triangleCount,
  };
}
