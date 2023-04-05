import {
  CopyObjectCommand,
  DeleteObjectsCommand,
  GetObjectCommand,
  ListObjectsV2Command,
} from "@aws-sdk/client-s3";
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

  // Begin fetching
  const modelPromise = fetchModel(projectId);
  const assetsPromise = fetchAssets(projectId);

  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, modelId } = paramsSchema.parse(params);

  // Verify user owns the publication
  const foundPublication = await prisma.publication.findFirst({
    where: { id, owner: session.address, PublishedModel: { id: modelId } },
  });
  if (!foundPublication) return new Response("Publication not found", { status: 404 });

  // Verify user owns the project
  const foundProject = await prisma.project.findFirst({
    where: { id: projectId, owner: session.address },
  });
  if (!foundProject) return new Response("Project not found", { status: 404 });

  // Get assets from S3 and model
  const [allAssets, assets] = await Promise.all([assetsPromise, getUsedAssets(await modelPromise)]);

  const usedAssets = allAssets.filter((asset) => assets.has(`/assets/${asset}`));
  const unusedAssets = allAssets.filter((asset) => !usedAssets.includes(asset));

  await Promise.all([
    // Delete unused assets from S3
    s3Client.send(
      new DeleteObjectsCommand({
        Bucket: env.S3_BUCKET,
        Delete: {
          Objects: unusedAssets.map((asset) => ({
            Key: S3Path.project(projectId).asset(asset),
          })),
        },
      })
    ),

    // Copy used assets to S3
    ...usedAssets.map((asset) =>
      s3Client.send(
        new CopyObjectCommand({
          Bucket: env.S3_BUCKET,
          CopySource: `${env.S3_BUCKET}/${S3Path.project(projectId).asset(asset)}`,
          Key: S3Path.publication(id).model(modelId).asset(asset),
          ACL: "public-read",
        })
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

async function fetchAssets(projectId: string) {
  const { Contents } = await s3Client.send(
    new ListObjectsV2Command({
      Bucket: env.S3_BUCKET,
      Prefix: S3Path.project(projectId).assets,
    })
  );

  const allAssets = Contents ?? [];
  const assetKeys = allAssets.map((asset) => asset.Key ?? "");
  const assetIds = assetKeys.map((asset) => asset.split("/").pop() ?? "");

  return assetIds;
}
