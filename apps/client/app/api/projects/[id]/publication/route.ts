import { DeleteObjectsCommand, GetObjectCommand } from "@aws-sdk/client-s3";
import { NextRequest, NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { getServerSession } from "@/src/server/helpers/getServerSession";
import { getUsedAssets } from "@/src/server/helpers/getUsedAssets";
import { prisma } from "@/src/server/prisma";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

import { Params, paramsSchema } from "../types";
import { PublishProjectResponse } from "./types";

// Publish project
export async function POST(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);

  // Begin fetching model
  const modelPromise = fetchModel(id);

  // Verify user is authenticated
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
    include: { Publication: true, ProjectAsset: true },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  let publicationId = found.Publication?.id;

  // Create a new publication if there is not one already
  if (!publicationId) {
    const newPublication = await prisma.publication.create({
      data: { owner: session.address },
      select: { id: true },
    });

    publicationId = newPublication.id;

    // Update project with new publication
    await prisma.project.update({ where: { id }, data: { publicationId } });
  }

  // Remove unused assets
  const model = await modelPromise;
  const assets = await getUsedAssets(model);
  const usedAssets = found.ProjectAsset.filter((asset) =>
    assets.has(S3Path.project(id).asset(asset.id))
  );
  const unusedAssets = found.ProjectAsset.filter((asset) => !usedAssets.includes(asset));

  await Promise.all([
    // Delete unused assets from S3
    s3Client.send(
      new DeleteObjectsCommand({
        Bucket: env.S3_BUCKET,
        Delete: {
          Objects: unusedAssets.map((asset) => ({
            Key: S3Path.project(id).asset(asset.id),
          })),
        },
      })
    ),

    // Delete unused assets from database
    prisma.projectAsset.deleteMany({
      where: {
        id: { in: unusedAssets.map((asset) => asset.id) },
      },
    }),
  ]);

  const json: PublishProjectResponse = { id: publicationId };
  return NextResponse.json(json);
}

async function fetchModel(projectId: string) {
  const Key = S3Path.project(projectId).model;
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });

  const { Body } = await s3Client.send(command);
  if (!Body) throw new Error("Model not found");

  return await Body.transformToByteArray();
}
