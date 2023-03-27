import { DeleteObjectsCommand, GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "../../../src/env.mjs";
import { s3Client } from "../../../src/server/s3";
import { S3Path } from "../../../src/utils/s3Paths";

export const expiresIn = 600; // 10 minutes

export const PROFILE_FILE = {
  IMAGE: "image",
  COVER: "cover",
  METADATA: "metadata",
} as const;

export type ProfileFile = (typeof PROFILE_FILE)[keyof typeof PROFILE_FILE];

export async function getUpload(id: number, type: ProfileFile) {
  const Key = S3Path.profile(id)[type];
  const ContentType = getContentType(type);
  const command = new PutObjectCommand({ Bucket: env.S3_BUCKET, Key, ContentType });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

export async function getDownload(id: number, type: ProfileFile) {
  const Key = S3Path.profile(id)[type];
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

export async function deleteFiles(id: number) {
  const paths = S3Path.profile(id);

  const command = new DeleteObjectsCommand({
    Bucket: env.S3_BUCKET,
    Delete: { Objects: Object.values(paths).map((Key) => ({ Key })) },
  });

  await s3Client.send(command);
}

export function getContentType(type: ProfileFile) {
  switch (type) {
    case PROFILE_FILE.IMAGE: {
      return "image/jpeg";
    }

    case PROFILE_FILE.COVER: {
      return "image/jpeg";
    }

    case PROFILE_FILE.METADATA: {
      return "application/json";
    }
  }
}
