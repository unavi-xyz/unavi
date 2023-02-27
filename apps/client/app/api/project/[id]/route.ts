import { NextResponse } from "next/server";

import { getServerSession } from "../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../src/server/prisma";
import { deleteFiles } from "../files";
import { Params, paramsSchema, patchSchema } from "./types";

// Update project
export async function PATCH(request: Request, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) throw new Error("Unauthorized");

  const { id } = paramsSchema.parse(params);
  const { name, description, publicationId } = patchSchema.parse(await request.json());

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
    include: { Publication: true },
  });
  if (!found) throw new Error("Not found");

  // Verify user owns the publication
  if (publicationId) {
    const publication = await prisma.publication.findFirst({
      where: { id: publicationId, owner: session.address },
    });
    if (!publication) throw new Error("Publication not found");
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
}

// Delete project
export async function DELETE(request: Request, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) throw new Error("Unauthorized");

  const { id } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
    include: { Publication: true },
  });
  if (!found) throw new Error("Not found");

  const publicationInUse = Boolean(found.Publication && found.Publication.spaceId !== null);

  await Promise.all([
    // Delete from database
    prisma.project.delete({ where: { id }, include: { Publication: !publicationInUse } }),
    // Delete files from S3
    deleteFiles(id),
  ]);

  return NextResponse.redirect("/create");
}
