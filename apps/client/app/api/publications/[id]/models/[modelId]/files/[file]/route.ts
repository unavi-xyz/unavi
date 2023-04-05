import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "../../../../../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../../../../../src/server/prisma";
import { getPublishedModelUpload } from "../../files";
import { GetFileUploadResponse, Params, paramsSchema } from "./types";

// Get file upload URL
export async function PUT(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, modelId, file } = paramsSchema.parse(params);

  // Verify user owns the publication, and the model exists
  const found = await prisma.publication.findFirst({
    where: { id, owner: session.address, PublishedModel: { id: modelId } },
  });
  if (!found) return new Response("Publication not found", { status: 404 });

  const url = await getPublishedModelUpload(id, modelId, file);

  const json: GetFileUploadResponse = { url };
  return NextResponse.json(json);
}
