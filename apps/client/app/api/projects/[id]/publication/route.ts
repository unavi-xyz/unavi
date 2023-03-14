import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "../../../../../src/server/helpers/getServerSession";
import { optimizeProject } from "../../../../../src/server/helpers/optimizeProject";
import { prisma } from "../../../../../src/server/prisma";
import { getContentType, getUpload } from "../../../publications/files";
import { Params, paramsSchema } from "../types";
import { PublishProjectResponse } from "./types";

// Publish project
export async function POST(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

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

  // Create optimized model
  const optimizedModel = await optimizeProject(id);

  // Upload optimized model to publication
  const uploadUrl = await getUpload(publicationId, "model");

  await fetch(uploadUrl, {
    method: "PUT",
    body: new Blob([optimizedModel], { type: getContentType("model") }),
    headers: {
      "Content-Type": getContentType("model"),
      "x-amz-acl": "public-read",
    },
  });

  const json: PublishProjectResponse = {
    id: publicationId,
    modelSize: optimizedModel.byteLength,
  };
  return NextResponse.json(json);
}
