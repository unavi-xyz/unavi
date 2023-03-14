import { GetObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";
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

import { getKey, PROJECT_FILE } from "../../../app/api/projects/files";
import createEncoderModule from "../../../public/scripts/draco_encoder";
import { env } from "../../env/server.mjs";
import { bytesToDisplay } from "../../utils/bytesToDisplay";
import { s3Client } from "../client";

const expiresIn = 600; // 10 minutes

/**
 * Compresses a project's model
 * @param id Project ID
 * @returns The compressed model
 */
export async function optimizeProject(id: string) {
  // Fetch model from S3
  const modelKey = getKey(id, PROJECT_FILE.MODEL);
  const modelCommand = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key: modelKey });
  const url = await getSignedUrl(s3Client, modelCommand, { expiresIn });
  const response = await fetch(url);
  const buffer = await response.arrayBuffer();
  const array = new Uint8Array(buffer);

  const start = performance.now();

  // Load model
  const io = new NodeIO()
    .registerExtensions(extensions)
    .registerDependencies({ "draco3d.encoder": await createEncoderModule() });

  const doc = await io.readBinary(array);

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
  let optimizedArray = array;

  try {
    optimizedArray = await io.writeBinary(doc);
  } catch (e) {
    console.warn("Failed to write model: ", e);
  }

  console.info(
    "⚙️ Optimized model:",
    bytesToDisplay(array.byteLength),
    "->",
    bytesToDisplay(optimizedArray.byteLength),
    `(-${Math.round((1 - optimizedArray.byteLength / array.byteLength) * 100)}%)`,
    `(${Math.round(performance.now() - start)}ms)`
  );

  return optimizedArray;
}
