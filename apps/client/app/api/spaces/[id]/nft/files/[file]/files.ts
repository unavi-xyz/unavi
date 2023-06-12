import { GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "@/src/env.mjs";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

const expiresIn = 600; // 10 minutes

export const SPACE_NFT_FILE = {
  metadata: "metadata",
} as const;

export type SpaceNFTFile = (typeof SPACE_NFT_FILE)[keyof typeof SPACE_NFT_FILE];

export async function getSpaceNFTUploadURL(id: string, type: SpaceNFTFile) {
  const Key = S3Path.spaceNFT(id)[type];
  const ContentType = getContentType(type);
  const command = new PutObjectCommand({
    ACL: "public-read",
    Bucket: env.S3_BUCKET,
    ContentType,
    Key,
  });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

export async function getSpaceNFTDownloadURL(id: string, type: SpaceNFTFile) {
  const Key = S3Path.spaceNFT(id)[type];
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });
  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}

function getContentType(type: SpaceNFTFile) {
  switch (type) {
    case SPACE_NFT_FILE.metadata: {
      return "application/json";
    }
  }
}
