import { DeleteObjectsCommand } from "@aws-sdk/client-s3";
import { NextRequest, NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { getServerSession } from "@/src/server/helpers/getServerSession";
import { listObjectsRecursive } from "@/src/server/helpers/listObjectsRecursive";
import { prisma } from "@/src/server/prisma";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

import { Params, paramsSchema } from "../types";
import { PostModelsResponse } from "./types";

// Create new publication model
export async function POST(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the publication
  const found = await prisma.publication.findFirst({
    where: { id, owner: session.address },
    include: { PublishedModel: true },
  });
  if (!found) return new Response("Publication not found", { status: 404 });

  // Remove existing model
  if (found.PublishedModel) {
    // Get all files within model directory
    const modelObjects = await listObjectsRecursive(
      S3Path.publication(id).model(found.PublishedModel.id).directory
    );

    const keys = modelObjects
      .map((object) => object.Key)
      .filter((key): key is string => key !== undefined);

    await Promise.all([
      // Remove from S3
      s3Client.send(
        new DeleteObjectsCommand({
          Bucket: env.S3_BUCKET,
          Delete: {
            Objects: keys.map((Key) => ({ Key })),
          },
        })
      ),

      // Remove from database
      prisma.publishedModel.delete({ where: { id: found.PublishedModel.id } }),
    ]);
  }

  // Create new model
  const newModel = await prisma.publishedModel.create({
    data: { publicationId: id },
    select: { id: true },
  });

  // Return new model id
  const json: PostModelsResponse = { modelId: newModel.id };
  return NextResponse.json(json);
}
