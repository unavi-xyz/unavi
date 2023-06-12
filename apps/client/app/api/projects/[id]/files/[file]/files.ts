import { GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "@/src/env.mjs";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

const expiresIn = 600; // 10 minutes

export const PROJECT_FILE = {
  IMAGE: "image",
  MODEL: "model",
} as const;

export type ProjectFile = (typeof PROJECT_FILE)[keyof typeof PROJECT_FILE];

export async function getProjectUploadURL(id: string, type: ProjectFile) {
  const Key = S3Path.project(id)[type];
  const ContentType = getContentType(type);
  const command = new PutObjectCommand({ Bucket: env.S3_BUCKET, ContentType, Key });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

export async function getProjectDownloadURL(id: string, type: ProjectFile) {
  const Key = S3Path.project(id)[type];
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

function getContentType(type: ProjectFile) {
  switch (type) {
    case PROJECT_FILE.IMAGE: {
      return "image/jpeg";
    }

    case PROJECT_FILE.MODEL: {
      return "model/gltf-binary";
    }
  }
}
