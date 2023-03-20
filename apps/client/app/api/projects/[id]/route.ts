import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../src/server/prisma";
import { deleteFiles } from "../files";
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
  const { name, description, publicationId } = patchSchema.parse(await request.json());

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
    data: { name, description, publicationId },
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
    include: { Publication: true, Assets: true },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  const publicationInUse = Boolean(found.Publication && found.Publication.spaceId !== null);

  await Promise.all([
    // Remove projectId from assets
    prisma.asset.updateMany({ where: { projectId: id }, data: { projectId: null } }),
    // Delete project, publication, and assets from database
    prisma.project.delete({
      where: { id },
      include: {
        Publication: !publicationInUse,
        Assets: {
          where: {
            OR: [
              { publicationId: null },
              {
                publicationId: !publicationInUse ? found.Publication?.id : undefined,
              },
            ],
          },
        },
      },
    }),
    // Delete files from S3
    deleteFiles(id),
  ]);

  return NextResponse.json({ success: true });
}
