import { GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";
import { NodeIO } from "@gltf-transform/core";
import { dedup, draco, resample, sparse, textureCompress, weld } from "@gltf-transform/functions";
import { BehaviorExtension, extensions } from "engine";
import sharp from "sharp";

import { getContentType, getKey, PROJECT_FILE } from "../../../app/api/projects/files";
import createEncoderModule from "../../../public/scripts/draco_encoder";
import { env } from "../../env/server.mjs";
import { bytesToDisplay } from "../../utils/bytesToDisplay";
import { s3Client } from "../client";

const expiresIn = 600; // 10 minutes
const MEGABYTE = 1024 * 1024;

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
  // Ignore large models, it takes too long
  if (array.byteLength < 30 * MEGABYTE) {
    try {
      await doc.transform(dedup(), weld(), resample(), sparse());
    } catch (e) {
      console.warn("Failed to optimize model: ", e);
    }
  }

  // Compress model
  await doc.transform(
    textureCompress({ encoder: sharp, targetFormat: "webp", resize: [4096, 4096] }),
    draco()
  );

  // Write model
  const optimizedArray = await io.writeBinary(doc);

  console.info(
    "⚙️ Optimized model:",
    bytesToDisplay(array.byteLength),
    "->",
    bytesToDisplay(optimizedArray.byteLength),
    `(-${Math.round((1 - optimizedArray.byteLength / array.byteLength) * 100)}%)`,
    `(${Math.round(performance.now() - start)}ms)`
  );

  // Upload model to S3
  const optimizedKey = getKey(id, PROJECT_FILE.OPTIMIZED_MODEL);
  const optimizedCommand = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: optimizedKey,
    Body: optimizedArray,
    ContentType: getContentType(PROJECT_FILE.OPTIMIZED_MODEL),
  });

  await s3Client.send(optimizedCommand);
}
