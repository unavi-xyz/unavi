import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "../../../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../../../src/server/prisma";
import { getAssetUpload } from "../s3";
import { Params, paramsSchema, PutAssetResponse } from "./types";

// Get asset upload URL
export async function PUT(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, assetId } = paramsSchema.parse(params);

  // Verify user owns the project, and the asset exists
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
    select: { Assets: { where: { id: assetId } } },
  });
  if (!found) return new Response("Not found", { status: 404 });

  const url = await getAssetUpload(assetId);

  const json: PutAssetResponse = { url };
  return NextResponse.json(json);
}
