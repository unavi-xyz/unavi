import { DeleteObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";
import { eq } from "drizzle-orm";
import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { auth } from "@/src/server/auth/lucia";
import { db } from "@/src/server/db/drizzle";
import { profile } from "@/src/server/db/schema";
import { FixWith } from "@/src/server/db/types";
import { nanoidShort } from "@/src/server/nanoid";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

import { GetFileUploadResponse, paramsSchema, ProfileFile } from "./types";

type Params = { params: { file: string } };

/**
 * Get S3 upload URL
 */
export async function PUT(request: NextRequest, { params }: Params) {
  const authRequest = auth.handleRequest({ cookies, request });
  const { session } = await authRequest.validateUser();
  if (!session) return new Response(null, { status: 401 });

  // Get user profile
  const _foundUser = await db.query.user.findFirst({
    where: (row, { eq }) => eq(row.id, session.userId),
    with: { profile: true },
  });
  if (!_foundUser) return new Response(null, { status: 404 });
  const foundUser: FixWith<typeof _foundUser, "profile"> = _foundUser;
  if (!foundUser.profile) return new Response(null, { status: 404 });

  const { file } = paramsSchema.parse(params);
  const idName = file === ProfileFile.image ? "imageKey" : "backgroundKey";

  // Generate file ID
  const fileId = nanoidShort();

  const getUploadURL = async () => {
    const command = new PutObjectCommand({
      Bucket: env.S3_BUCKET,
      ContentType: "image/*",
      Key: S3Path.profile(session.userId)[file](fileId),
    });
    const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
    return url;
  };

  const removePreviousFile = async () => {
    if (!foundUser.profile) return;

    const prevId = foundUser.profile[idName];
    if (!prevId) return;

    const command = new DeleteObjectCommand({
      Bucket: env.S3_BUCKET,
      Key: S3Path.profile(session.userId)[file](prevId),
    });

    await s3Client.send(command);
  };

  const [url] = await Promise.all([
    getUploadURL(),
    removePreviousFile(),
    db
      .update(profile)
      .set({ [idName]: fileId })
      .where(eq(profile.userId, session.userId)),
  ]);

  const json: GetFileUploadResponse = { fileId, url };
  return NextResponse.json(json);
}
