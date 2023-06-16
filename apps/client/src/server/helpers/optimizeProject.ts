import { NodeIO } from "@gltf-transform/core";
import { draco, textureCompress } from "@gltf-transform/functions";
import { extensions, optimizeDocument } from "@unavi/engine";
import sharp from "sharp";

import createEncoderModule from "@/public/scripts/draco_encoder";

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

  // Optimize model
  await optimizeDocument(doc);

  // Compress model
  try {
    await doc.transform(
      textureCompress({
        encoder: sharp,
        resize: [4096, 4096],
        targetFormat: "webp",
      }),
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
    `(-${Math.round(
      (1 - optimizedModel.byteLength / model.byteLength) * 100
    )}%)`,
    `(${Math.round(performance.now() - start)}ms)`
  );

  return optimizedModel;
}
