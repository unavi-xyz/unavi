import { DeleteObjectsCommand, GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "@/src/env.mjs";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

export const expiresIn = 600; // 10 minutes

export const PUBLISHED_MODEL_FILE = {
  IMAGE: "image",
  MODEL: "model",
} as const;

export type PublishedModelFile = (typeof PUBLISHED_MODEL_FILE)[keyof typeof PUBLISHED_MODEL_FILE];

export async function getPublishedModelUpload(
  publicationId: string,
  modelId: string,
  type: PublishedModelFile
) {
  const Key = S3Path.publication(publicationId).model(modelId)[type];
  const ContentType = getContentType(type);
  const command = new PutObjectCommand({ Bucket: env.S3_BUCKET, Key, ContentType });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

export async function getPublishedModelDownload(
  publicationId: string,
  modelId: string,
  type: PublishedModelFile
) {
  const Key = S3Path.publication(publicationId).model(modelId)[type];
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

export async function deletePublishedModelFiles(publicationId: string, modelId: string) {
  const paths = S3Path.publication(publicationId).model(modelId);

  const command = new DeleteObjectsCommand({
    Bucket: env.S3_BUCKET,
    Delete: {
      Objects: [{ Key: paths.image }, { Key: paths.model }],
    },
  });

  await s3Client.send(command);
}

export function getContentType(type: PublishedModelFile) {
  switch (type) {
    case PUBLISHED_MODEL_FILE.IMAGE: {
      return "image/jpg";
    }

    case PUBLISHED_MODEL_FILE.MODEL: {
      return "model/gltf-binary";
    }
  }
}
