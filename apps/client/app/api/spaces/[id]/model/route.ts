import { DeleteObjectsCommand } from "@aws-sdk/client-s3";
import { NextRequest, NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { getUserSession } from "@/src/server/auth/getUserSession";
import { listObjectsRecursive } from "@/src/server/helpers/listObjectsRecursive";
import { nanoidShort } from "@/src/server/nanoid";
import { prisma } from "@/src/server/prisma";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

import { Params, paramsSchema } from "../types";
import { PostSpaceModelResponse } from "./types";

// Create new space model
export async function POST(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session || !session.user.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({
    where: { publicId: id, owner: session.user.address },
    include: { SpaceModel: true },
  });
  if (!found) return new Response("Space not found", { status: 404 });

  // Remove existing space model
  if (found.SpaceModel) {
    const allObjects = await listObjectsRecursive(
      S3Path.spaceModel(found.SpaceModel.publicId).directory
    );

    await Promise.all([
      // Remove objects from S3
      allObjects.length > 0
        ? s3Client.send(
            new DeleteObjectsCommand({
              Bucket: env.S3_BUCKET,
              Delete: { Objects: allObjects.map((obj) => ({ Key: obj.Key })) },
            })
          )
        : null,

      // Remove from database
      prisma.spaceModel.delete({ where: { id: found.SpaceModel.id } }),
    ]);
  }

  // Create new space model
  const model = await prisma.spaceModel.create({
    data: { publicId: nanoidShort(), spaceId: found.id },
  });

  const json: PostSpaceModelResponse = { modelId: model.publicId };
  return NextResponse.json(json);
}
