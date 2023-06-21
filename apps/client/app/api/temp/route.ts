import { PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";
import { NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { nanoidShort } from "@/src/server/nanoid";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

import { GetTempResponse } from "./types";

export const runtime = "edge";
export const preferredRegion = "iad1";

const expiresIn = 600; // 10 minutes

// Get temp upload URL
export async function POST() {
  const fileId = nanoidShort();

  const command = new PutObjectCommand({
    ACL: "public-read",
    Bucket: env.S3_BUCKET,
    Key: S3Path.temp(fileId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn });

  const json: GetTempResponse = { fileId, url };
  return NextResponse.json(json);
}
