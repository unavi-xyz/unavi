import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../src/server/prisma";
import { deleteFiles } from "../files";
import { Params, paramsSchema } from "./types";

// Delete publication
export async function DELETE(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the publication
  const found = await prisma.publication.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) return new Response("Publication not found", { status: 404 });

  await Promise.all([
    // Delete from database
    prisma.publication.delete({ where: { id }, include: { ViewEvents: true } }),
    // Delete files from S3
    deleteFiles(id),
  ]);

  return NextResponse.redirect("/create");
}
