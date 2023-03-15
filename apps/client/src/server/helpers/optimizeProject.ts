import { NodeIO } from "@gltf-transform/core";
import {
  dedup,
  draco,
  metalRough,
  resample,
  sparse,
  textureCompress,
} from "@gltf-transform/functions";
import { BehaviorExtension, extensions } from "engine";
import sharp from "sharp";

import createEncoderModule from "../../../public/scripts/draco_encoder";
import { bytesToDisplay } from "../../utils/bytesToDisplay";

/**
 * Compresses a project's model
 * @param id Project ID
 * @returns The compressed model
 */
export async function optimizeModel(model: Uint8Array) {
  const start = performance.now();

  // Load model
  const io = new NodeIO()
    .registerExtensions(extensions)
    .registerDependencies({ "draco3d.encoder": await createEncoderModule() });

  const doc = await io.readBinary(model);

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
    await doc.transform(dedup(), metalRough(), resample(), sparse());
  } catch (err) {
    console.warn("Failed to optimize model.");
    console.warn(err);
  }

  // Compress model
  try {
    await doc.transform(
      textureCompress({ encoder: sharp, targetFormat: "webp", resize: [4096, 4096] }),
      draco()
    );
  } catch (e) {
    console.warn("Failed to compress model: ", e);
  }

  // Write model
  let optimizedModel = model;

  try {
    optimizedModel = await io.writeBinary(doc);
  } catch (e) {
    console.warn("Failed to write model: ", e);
  }

  console.info(
    "⚙️ Optimized model:",
    bytesToDisplay(model.byteLength),
    "->",
    bytesToDisplay(optimizedModel.byteLength),
    `(-${Math.round((1 - optimizedModel.byteLength / model.byteLength) * 100)}%)`,
    `(${Math.round(performance.now() - start)}ms)`
  );

  return optimizedModel;
}
