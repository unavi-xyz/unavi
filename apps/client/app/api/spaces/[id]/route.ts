import { DeleteObjectsCommand } from "@aws-sdk/client-s3";
import { NextRequest, NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { getUserSession } from "@/src/server/auth/getUserSession";
import { listObjectsRecursive } from "@/src/server/helpers/listObjectsRecursive";
import { prisma } from "@/src/server/prisma";
import { s3Client } from "@/src/server/s3";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { GetSpaceResponse, Params, paramsSchema, patchSchema } from "./types";

// Get space
export async function GET(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);

  const space = await prisma.space.findFirst({
    include: { SpaceModel: true },
    where: { publicId: id },
  });
  if (!space) return new Response("Space not found", { status: 404 });

  const modelURI = space.SpaceModel
    ? cdnURL(S3Path.spaceModel(space.SpaceModel.publicId).model)
    : null;

  const json: GetSpaceResponse = { ownerId: space.ownerId, uri: modelURI };
  return NextResponse.json(json);
}

// Update space
export async function PATCH(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({
    include: { SpaceModel: true },
    where: { ownerId: session.user.userId, publicId: id },
  });
  if (!found) return new Response("Space not found", { status: 404 });

  const { tokenId } = patchSchema.parse(await request.json());

  // Update space
  await prisma.space.update({ data: { tokenId }, where: { id: found.id } });

  return NextResponse.json({ success: true });
}

// Delete space
export async function DELETE(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({
    include: { SpaceModel: true, SpaceNFT: true },
    where: { ownerId: session.user.userId, publicId: id },
  });
  if (!found) return new Response("Space not found", { status: 404 });

  // Delete files from S3
  const objectsPromise = Promise.all([
    found.SpaceModel
      ? await listObjectsRecursive(S3Path.spaceModel(found.SpaceModel.publicId).directory)
      : [],
    found.SpaceNFT
      ? await listObjectsRecursive(S3Path.spaceNFT(found.SpaceNFT.publicId).directory)
      : [],
  ])
    .then((results) => results.flat())
    .then((objs) =>
      objs.length > 0
        ? s3Client.send(
            new DeleteObjectsCommand({
              Bucket: env.S3_BUCKET,
              Delete: { Objects: objs.map(({ Key }) => ({ Key })) },
            })
          )
        : null
    );

  await Promise.all([
    found.SpaceModel ? prisma.spaceModel.delete({ where: { id: found.SpaceModel.id } }) : null,
    found.SpaceNFT ? prisma.spaceNFT.delete({ where: { id: found.SpaceNFT.id } }) : null,
    // Disconnect projects from space
    prisma.project.updateMany({ data: { spaceId: null }, where: { spaceId: found.id } }),
  ]);

  // Delete space from database
  await prisma.space.delete({ where: { id: found.id } });

  await objectsPromise;

  return NextResponse.json({ success: true });
}
