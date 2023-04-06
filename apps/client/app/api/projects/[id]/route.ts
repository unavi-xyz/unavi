import { DeleteObjectsCommand } from "@aws-sdk/client-s3";
import { NextRequest, NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { getServerSession } from "@/src/server/helpers/getServerSession";
import { listObjectsRecursive } from "@/src/server/helpers/listObjectsRecursive";
import { prisma } from "@/src/server/prisma";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

import { Params, paramsSchema, patchSchema } from "./types";

// Get project
export async function GET(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the project
  const project = await prisma.project.findFirst({
    where: { id, owner: session.address },
    include: { Publication: true },
  });
  if (!project) return new Response("Project not found", { status: 404 });

  return NextResponse.json(project);
}

// Update project
export async function PATCH(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);
  const { title, description, publicationId } = patchSchema.parse(await request.json());

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
    include: { Publication: true },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  // Verify user owns the publication
  if (publicationId) {
    const publication = await prisma.publication.findFirst({
      where: { id: publicationId, owner: session.address },
    });
    if (!publication) return new Response("Publication not found", { status: 404 });
  }

  // Delete old publication if not in use
  if (publicationId !== undefined) {
    if (found.Publication?.spaceId === null) {
      await prisma.publication.delete({ where: { id: found.Publication.id } });
    }
  }

  // Update project
  await prisma.project.update({
    where: { id },
    data: { title, description, publicationId },
  });

  return NextResponse.json({ success: true });
}

// Delete project
export async function DELETE(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
    include: { Publication: true },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  const publicationInUse = Boolean(found.Publication && found.Publication.spaceId !== null);

  // Get objects from S3
  // If publication is in use, don't delete it
  const [projectObjects, publicationObjects] = await Promise.all([
    listObjectsRecursive(S3Path.project(id).directory),
    publicationInUse ? Promise.resolve([]) : listObjectsRecursive(S3Path.publication(id).directory),
  ]);

  const allObjects = [...projectObjects, ...publicationObjects];

  await Promise.all([
    // Delete objects from S3
    s3Client.send(
      new DeleteObjectsCommand({
        Bucket: env.S3_BUCKET,
        Delete: {
          Objects: allObjects.map(({ Key }) => ({ Key })),
        },
      })
    ),

    // Delete project and publication from database
    // If publication is in use, don't delete it
    prisma.project.delete({
      where: { id },
      include: {
        Publication: !publicationInUse,
      },
    }),
  ]);

  return NextResponse.json({ success: true });
}
