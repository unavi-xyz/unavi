import { DeleteObjectCommand, GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { NextRequest, NextResponse } from "next/server";

import { env } from "../../../../../src/env.mjs";
import { getServerSession } from "../../../../../src/server/helpers/getServerSession";
import { getUsedAssets } from "../../../../../src/server/helpers/getUsedAssets";
import { optimizeModel } from "../../../../../src/server/helpers/optimizeProject";
import { prisma } from "../../../../../src/server/prisma";
import { s3Client } from "../../../../../src/server/s3";
import { cdnURL, S3Path } from "../../../../../src/utils/s3Paths";
import { getContentType } from "../../../publications/files";
import { Params, paramsSchema } from "../types";
import { postSchema, PublishProjectResponse } from "./types";

// Publish project
export async function POST(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);

  // Begin fetching model
  const modelPromise = fetchModel(id);

  // Verify user is authenticated
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { optimize } = postSchema.parse(await request.json());

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
    include: { Publication: true, Assets: { select: { id: true, PublicationAsset: true } } },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  // Create new publication if it doesn't exist
  let publicationId = found.Publication?.id;

  if (!publicationId) {
    const newPublication = await prisma.publication.create({
      data: { owner: session.address },
      select: { id: true },
    });

    publicationId = newPublication.id;

    // Update project
    await prisma.project.update({ where: { id }, data: { publicationId } });
  }

  const model = await modelPromise;
  let publishedModel = model;

  const assets = await getUsedAssets(model);
  const usedAssets = found.Assets.filter((asset) => assets.has(cdnURL(S3Path.asset(asset.id))));
  const unusedAssets = found.Assets.filter((asset) => {
    return (
      !assets.has(cdnURL(S3Path.asset(asset.id))) &&
      (asset.PublicationAsset.length === 0 ||
        (asset.PublicationAsset.length === 1 &&
          asset.PublicationAsset[0]?.publicationId === publicationId))
    );
  });

  await Promise.all([
    // Delete unused assets from S3
    Promise.all(
      unusedAssets.map((asset) => {
        const command = new DeleteObjectCommand({
          Bucket: env.S3_BUCKET,
          Key: S3Path.asset(asset.id),
        });
        return s3Client.send(command);
      })
    ),

    prisma.$transaction([
      // Delete unused assets from database
      prisma.asset.deleteMany({
        where: {
          id: { in: unusedAssets.map((asset) => asset.id) },
        },
      }),
      prisma.publicationAsset.deleteMany({
        where: {
          assetId: { in: unusedAssets.map((asset) => asset.id) },
        },
      }),
      // Update used assets publicationId
      prisma.asset.updateMany({
        where: {
          id: { in: usedAssets.map((asset) => asset.id) },
        },
        data: {},
      }),
    ]),
  ]);

  // Optimize model
  if (optimize) {
    try {
      publishedModel = await optimizeModel(model);
    } catch (error) {
      console.error("Failed to process model", error);
    }
  }

  // Upload model to publication bucket
  const Key = S3Path.publication(publicationId).model;
  const ContentType = getContentType("model");
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key,
    ContentType,
    Body: publishedModel,
    ACL: "public-read",
  });

  await s3Client.send(command);

  const json: PublishProjectResponse = {
    id: publicationId,
    modelSize: publishedModel.byteLength,
  };
  return NextResponse.json(json);
}

async function fetchModel(projectId: string) {
  const Key = S3Path.project(projectId).model;
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });

  const { Body } = await s3Client.send(command);
  if (!Body) throw new Error("Model not found");

  return await Body.transformToByteArray();
}
