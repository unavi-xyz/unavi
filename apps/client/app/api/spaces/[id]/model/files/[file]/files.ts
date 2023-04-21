import { GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "@/src/env.mjs";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

const expiresIn = 600; // 10 minutes

export const SPACE_MODEL_FILE = {
  IMAGE: "image",
  METADATA: "metadata",
  MODEL: "model",
} as const;

export type SpaceModelFile = (typeof SPACE_MODEL_FILE)[keyof typeof SPACE_MODEL_FILE];

export async function getSpaceModelUploadURL(id: string, type: SpaceModelFile) {
  const Key = S3Path.spaceModel(id)[type];
  const ContentType = getContentType(type);
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key,
    ContentType,
    ACL: "public-read",
  });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

export async function getSpaceModelDownloadURL(id: string, type: SpaceModelFile) {
  const Key = S3Path.spaceModel(id)[type];
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

function getContentType(type: SpaceModelFile) {
  switch (type) {
    case SPACE_MODEL_FILE.IMAGE: {
      return "image/jpeg";
    }

    case SPACE_MODEL_FILE.METADATA: {
      return "application/json";
    }

    case SPACE_MODEL_FILE.MODEL: {
      return "model/gltf-binary";
    }
  }
}
