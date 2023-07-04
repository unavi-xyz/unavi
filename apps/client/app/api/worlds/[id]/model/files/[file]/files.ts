import {
  GetObjectCommand,
  PutObjectCommand,
  PutObjectCommandInput,
} from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "@/src/env.mjs";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

const expiresIn = 600; // 10 minutes

export const WORLD_MODEL_FILE = {
  IMAGE: "image",
  MODEL: "model",
} as const;

export type WorldModelFile =
  (typeof WORLD_MODEL_FILE)[keyof typeof WORLD_MODEL_FILE];

export function getWorldModelFileUploadCommand(
  publicId: string,
  type: WorldModelFile,
  Body?: PutObjectCommandInput["Body"]
) {
  const Key = S3Path.worldModel(publicId)[type];
  const ContentType = getContentType(type);
  const command = new PutObjectCommand({
    ACL: "public-read",
    Body,
    Bucket: env.S3_BUCKET,
    ContentType,
    Key,
  });
  return command;
}

export async function getWorldModelUploadURL(
  publicId: string,
  type: WorldModelFile
) {
  const command = getWorldModelFileUploadCommand(publicId, type);
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

export async function getWorldModelDownloadURL(
  id: string,
  type: WorldModelFile
) {
  const Key = S3Path.worldModel(id)[type];
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

function getContentType(type: WorldModelFile) {
  switch (type) {
    case WORLD_MODEL_FILE.IMAGE: {
      return "image/jpeg";
    }

    case WORLD_MODEL_FILE.MODEL: {
      return "model/gltf-binary";
    }
  }
}
