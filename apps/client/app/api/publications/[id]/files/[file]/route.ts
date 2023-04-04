import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { prisma } from "@/src/server/prisma";

import { getPublicationUpload } from "../../files";
import { Params } from "../../types";
import { GetFileUploadResponse, paramsSchema } from "./types";

// Get file upload URL
export async function PUT(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the publication
  const found = await prisma.publication.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  const url = await getPublicationUpload(id, file);

  const json: GetFileUploadResponse = { url };
  return NextResponse.json(json);
}
