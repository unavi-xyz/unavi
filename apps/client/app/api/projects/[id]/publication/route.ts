import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { prisma } from "@/src/server/prisma";

import { Params, paramsSchema } from "../types";
import { PublishProjectResponse } from "./types";

// Publish project
export async function POST(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);

  // Verify user is authenticated
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
    include: { Publication: true, ProjectAsset: true },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  let publicationId = found.Publication?.id;

  // Create a new publication if there is not one already
  if (!publicationId) {
    const newPublication = await prisma.publication.create({
      data: { owner: session.address },
      select: { id: true },
    });

    publicationId = newPublication.id;

    // Update project with new publication
    await prisma.project.update({ where: { id }, data: { publicationId } });
  }

  const json: PublishProjectResponse = { id: publicationId };
  return NextResponse.json(json);
}
