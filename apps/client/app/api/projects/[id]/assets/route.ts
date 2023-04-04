import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { prisma } from "@/src/server/prisma";

import { getAssetUpload } from "./s3";
import { paramsSchema, PostAssetsResponse } from "./types";
import { Params } from "./types";

// Get new asset upload URL
export async function POST(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  // Create new asset
  const asset = await prisma.projectAsset.create({
    data: { projectId: id },
  });

  const url = await getAssetUpload(id, asset.id);

  const json: PostAssetsResponse = { url, assetId: asset.id };
  return NextResponse.json(json);
}
