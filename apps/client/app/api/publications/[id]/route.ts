import { NextRequest } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { prisma } from "@/src/server/prisma";

import { deletePublicationFiles } from "./files";
import { Params, paramsSchema } from "./types";

// Delete publication
export async function DELETE(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the publication
  const found = await prisma.publication.findFirst({ where: { id, owner: session.address } });
  if (!found) return new Response("Publication not found", { status: 404 });

  await Promise.all([
    // Delete files from S3
    deletePublicationFiles(id),
    // Delete publication from database
    prisma.publication.delete({ where: { id } }),
  ]);
}
