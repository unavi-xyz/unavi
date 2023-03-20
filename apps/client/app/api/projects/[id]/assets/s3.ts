import { GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { pathAsset } from "../../../../../src/editor/utils/s3Paths";
import { env } from "../../../../../src/env/server.mjs";
import { s3Client } from "../../../../../src/server/client";

const expiresIn = 600; // 10 minutes

export async function getAssetDownload(assetId: string) {
  const Key = pathAsset(assetId);
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

export async function getAssetUpload(assetId: string) {
  const Key = pathAsset(assetId);
  const command = new PutObjectCommand({ Bucket: env.S3_BUCKET, Key });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}
