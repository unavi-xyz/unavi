import { PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";
import { NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { nanoidShort } from "@/src/server/nanoid";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

export const runtime = "edge";

const expiresIn = 600; // 10 minutes

// Get temp upload URL
export async function POST() {
  const fileId = nanoidShort();

  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: S3Path.temp(fileId),
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn });

  return NextResponse.json({ url, fileId });
}
