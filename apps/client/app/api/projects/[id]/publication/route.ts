import { GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { NextRequest, NextResponse } from "next/server";

import { env } from "../../../../../src/env/server.mjs";
import { s3Client } from "../../../../../src/server/client";
import { getServerSession } from "../../../../../src/server/helpers/getServerSession";
import { optimizeModel } from "../../../../../src/server/helpers/optimizeProject";
import { prisma } from "../../../../../src/server/prisma";
import { getContentType, getKey } from "../../../publications/files";
import { PROJECT_FILE } from "../../files";
import { Params, paramsSchema } from "../types";
import { postSchema, PublishProjectResponse } from "./types";

// Publish project
export async function POST(request: NextRequest, { params }: Params) {
  // Verify user is authenticated
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { optimize } = postSchema.parse(await request.json());
  const { id } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
    include: { Publication: true },
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

  // Fetch model
  const model = await fetchModel(publicationId);
  let publishedModel = model;

  // Optimize model
  if (optimize) {
    try {
      publishedModel = await optimizeModel(model);
    } catch (error) {
      console.error("Failed to process model", error);
    }
  }

  // Upload model to publication bucket
  const Key = getKey(publicationId, "model");
  const ContentType = getContentType("model");
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key,
    ContentType,
    Body: publishedModel,
  });

  await s3Client.send(command);

  const json: PublishProjectResponse = {
    id: publicationId,
    modelSize: publishedModel.byteLength,
  };
  return NextResponse.json(json);
}

async function fetchModel(publicationId: string) {
  const modelKey = getKey(publicationId, PROJECT_FILE.MODEL);
  const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key: modelKey });

  const { Body } = await s3Client.send(command);
  if (!Body) throw new Error("Model not found");

  return await Body.transformToByteArray();
}
