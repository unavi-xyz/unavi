import { CopyObjectCommand, DeleteObjectsCommand, GetObjectCommand } from "@aws-sdk/client-s3";
import { NextRequest } from "next/server";

import { env } from "@/src/env.mjs";
import { getServerSession } from "@/src/server/helpers/getServerSession";
import { getUsedAssets } from "@/src/server/helpers/getUsedAssets";
import { prisma } from "@/src/server/prisma";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

import { Params } from "../../../types";
import { paramsSchema, postSchema } from "./types";

// Copy project files to model
export async function POST(request: NextRequest, { params }: Params) {
  const json = await request.json();
  const { projectId } = postSchema.parse(json);

  // Begin fetching model
  const modelPromise = fetchModel(projectId);

  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, modelId } = paramsSchema.parse(params);

  // Verify user owns the publication and project
  const foundPublication = await prisma.publication.findFirst({
    where: { id, owner: session.address, PublishedModel: { id: modelId } },
  });
  if (!foundPublication) return new Response("Publication not found", { status: 404 });

  const foundProject = await prisma.project.findFirst({
    where: { id: projectId, owner: session.address },
    include: { ProjectAsset: true },
  });
  if (!foundProject) return new Response("Project not found", { status: 404 });

  // Get used assets
  const assets = await getUsedAssets(await modelPromise);
  const usedAssets = foundProject.ProjectAsset.filter((asset) => assets.has(`/assets/${asset.id}`));
  const unusedAssets = foundProject.ProjectAsset.filter((asset) => !usedAssets.includes(asset));

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

    // Copy used assets to S3
    Promise.all(
      usedAssets.map((asset) =>
        s3Client.send(
          new CopyObjectCommand({
            Bucket: env.S3_BUCKET,
            CopySource: `${env.S3_BUCKET}/${S3Path.project(projectId).asset(asset.id)}`,
            Key: S3Path.publication(id).model(modelId).asset(asset.id),
            ACL: "public-read",
          })
        )
      )
    ),
  ]);

  return new Response("OK", { status: 200 });
}

async function fetchModel(projectId: string) {
  const Key = S3Path.project(projectId).model;
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });

  const { Body } = await s3Client.send(command);
  if (!Body) throw new Error("Model not found");

  return await Body.transformToByteArray();
}
