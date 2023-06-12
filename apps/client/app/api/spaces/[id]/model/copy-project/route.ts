import {
  CopyObjectCommand,
  DeleteObjectsCommand,
  GetObjectCommand,
  ListObjectsV2Command,
} from "@aws-sdk/client-s3";
import { NextRequest } from "next/server";

import { env } from "@/src/env.mjs";
import { getUserSession } from "@/src/server/auth/getUserSession";
import { getUsedAssets } from "@/src/server/helpers/getUsedAssets";
import { nanoidShort } from "@/src/server/nanoid";
import { prisma } from "@/src/server/prisma";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

import { Params, paramsSchema } from "../../types";
import { postSchema } from "./types";

// Copy project files to model
export async function POST(request: NextRequest, { params }: Params) {
  const json = await request.json();
  const { projectId } = postSchema.parse(json);

  // Begin fetching
  const modelPromise = fetchModel(projectId);
  const assetsPromise = fetchAssets(projectId);

  const session = await getUserSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the space
  const foundSpace = await prisma.space.findFirst({
    include: { SpaceModel: true },
    where: { ownerId: session.user.userId, publicId: id },
  });
  if (!foundSpace) return new Response("Space not found", { status: 404 });

  // Verify user owns the project
  const foundProject = await prisma.project.findFirst({
    where: { ownerId: session.user.userId, publicId: projectId },
  });
  if (!foundProject) return new Response("Project not found", { status: 404 });

  let spaceModelId = foundSpace.SpaceModel?.publicId;

  // If no space model, create one
  if (spaceModelId === undefined) {
    const publicId = nanoidShort();
    await prisma.spaceModel.create({ data: { publicId, spaceId: foundSpace.id } });
    spaceModelId = publicId;
  }

  // Get assets from S3 and model
  const [allAssets, assets] = await Promise.all([assetsPromise, getUsedAssets(await modelPromise)]);
  const usedAssets = allAssets.filter((asset) => assets.has(`/assets/${asset}`));
  const unusedAssets = allAssets.filter((asset) => !usedAssets.includes(asset));

  await Promise.all([
    // Delete unused assets from S3
    unusedAssets.length > 0
      ? s3Client.send(
          new DeleteObjectsCommand({
            Bucket: env.S3_BUCKET,
            Delete: {
              Objects: unusedAssets.map((asset) => ({
                Key: S3Path.project(projectId).asset(asset),
              })),
            },
          })
        )
      : undefined,

    // Copy used assets to S3
    ...usedAssets.map((asset) => {
      if (spaceModelId === undefined) throw new Error("Space model id not found");

      return s3Client.send(
        new CopyObjectCommand({
          ACL: "public-read",
          Bucket: env.S3_BUCKET,
          CopySource: `${env.S3_BUCKET}/${S3Path.project(projectId).asset(asset)}`,
          Key: S3Path.spaceModel(spaceModelId).asset(asset),
        })
      );
    }),
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
