import { PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "../../env/server.mjs";
import { s3Client } from "./client";
import { expiresIn } from "./constants";

export async function getTempUpload(fileId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: `temp/${fileId}`,
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn });
  return url;
}
