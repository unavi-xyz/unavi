import { GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "../../../src/env/server.mjs";
import { s3Client } from "../../../src/server/s3/client";

export const expiresIn = 600; // 10 minutes

export const PROJECT_FILE = {
  IMAGE: "image",
  MODEL: "model",
  OPTIMIZED_MODEL: "optimized_model",
} as const;

export type ProjectFile = (typeof PROJECT_FILE)[keyof typeof PROJECT_FILE];

export async function getUpload(id: string, type: ProjectFile) {
  const Key = getKey(id, type);
  const ContentType = getContentType(type);
  const command = new PutObjectCommand({ Bucket: env.S3_BUCKET, Key, ContentType });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

export async function getDownload(id: string, type: ProjectFile) {
  const Key = getKey(id, type);
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

function getKey(id: string, type: ProjectFile) {
  switch (type) {
    case PROJECT_FILE.IMAGE: {
      return `projects/${id}/image.jpg`;
    }

    case PROJECT_FILE.MODEL: {
      return `projects/${id}/model.glb`;
    }

    case PROJECT_FILE.OPTIMIZED_MODEL: {
      return `projects/${id}/optimized_model.glb`;
    }
  }
}

function getContentType(type: ProjectFile) {
  switch (type) {
    case PROJECT_FILE.IMAGE: {
      return "image/jpeg";
    }

    case PROJECT_FILE.MODEL:
    case PROJECT_FILE.OPTIMIZED_MODEL: {
      return "model/gltf-binary";
    }
  }
}
