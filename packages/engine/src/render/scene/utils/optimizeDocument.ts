import { Document } from "@gltf-transform/core";
import { dedup, metalRough, resample, sparse, weld } from "@gltf-transform/functions";
import { BehaviorExtension } from "@wired-labs/gltf-extensions";

/**
 * Optimizes a glTF document. Does not apply compression.
 * @param doc The document to optimize.
 */
export async function optimizeDocument(doc: Document) {
  // Remove extras
  doc
    .getRoot()
    .listNodes()
    .forEach((node) => node.setExtras({}));

  doc
    .getRoot()
    .listMeshes()
    .forEach((mesh) => {
      mesh.listPrimitives().forEach((primitive) => primitive.setExtras({}));
      mesh.setExtras({});
    });

  doc
    .getRoot()
    .listExtensionsUsed()
    .forEach((extension) => {
      if (extension instanceof BehaviorExtension) {
        extension.listBehaviorNodes().forEach((behaviorNode) => behaviorNode.setExtras({}));
      }
    });

  // Optimize model
  try {
    await doc.transform(dedup(), weld(), metalRough(), resample(), sparse());
  } catch (err) {
    console.warn("Failed to optimize model.");
    console.warn(err);
  }
}
