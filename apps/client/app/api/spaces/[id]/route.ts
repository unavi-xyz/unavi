import { DeleteObjectsCommand } from "@aws-sdk/client-s3";
import { NextRequest, NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { getServerSession } from "@/src/server/helpers/getServerSession";
import { listObjectsRecursive } from "@/src/server/helpers/listObjectsRecursive";
import { prisma } from "@/src/server/prisma";
import { s3Client } from "@/src/server/s3";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { GetSpaceResponse, Params, paramsSchema, patchSchema } from "./types";

// Get space
export async function GET(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);

  const space = await prisma.space.findFirst({
    where: { publicId: id },
    include: { SpaceModel: true },
  });
  if (!space) return new Response("Space not found", { status: 404 });

  const modelURI = space.SpaceModel
    ? cdnURL(S3Path.spaceModel(space.SpaceModel.publicId).model)
    : null;

  const json: GetSpaceResponse = {
    owner: space.owner,
    uri: modelURI,
  };
  return NextResponse.json(json);
}

// Update space
export async function PATCH(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({
    where: { publicId: id, owner: session.address },
    include: { SpaceModel: true },
  });
  if (!found) return new Response("Space not found", { status: 404 });

  const { tokenId } = patchSchema.parse(await request.json());

  // Update space
  await prisma.space.update({ where: { id: found.id }, data: { tokenId } });

  return NextResponse.json({ success: true });
}

// Delete space
export async function DELETE(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({
    where: { publicId: id, owner: session.address },
    include: { SpaceModel: true },
  });
  if (!found) return new Response("Space not found", { status: 404 });

  const allObjects = found.SpaceModel
    ? await listObjectsRecursive(S3Path.spaceModel(found.SpaceModel.publicId).directory)
    : [];

  await Promise.all([
    // Delete files from S3
    allObjects.length > 0
      ? s3Client.send(
          new DeleteObjectsCommand({
            Bucket: env.S3_BUCKET,
            Delete: { Objects: allObjects.map(({ Key }) => ({ Key })) },
          })
        )
      : null,

    // Delete space from database
    prisma.space.delete({ where: { id: found.id }, include: { SpaceModel: true, SpaceNFT: true } }),
  ]);

  return NextResponse.json({ success: true });
}
