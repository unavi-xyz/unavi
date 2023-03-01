import { NextResponse } from "next/server";

import { getServerSession } from "../../../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../../../src/server/prisma";
import { getUpload } from "../../../files";
import { Params, paramsSchema } from "../types";
import { GetFileUploadResponse } from "./types";

// Get file upload URL
export async function GET(request: Request, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the publication
  const found = await prisma.publication.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  const url = await getUpload(id, file);

  const json: GetFileUploadResponse = { url };
  return NextResponse.json(json);
}
