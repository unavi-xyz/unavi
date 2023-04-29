import { NextRequest, NextResponse } from "next/server";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { nanoidShort } from "@/src/server/nanoid";
import { prisma } from "@/src/server/prisma";

import { getAssetUpload } from "./s3";
import { paramsSchema, PostAssetsResponse } from "./types";
import { Params } from "./types";

// Get new asset upload URL
export async function POST(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { publicId: id, ownerId: session.user.userId },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  // Create asset upload URL
  const assetId = nanoidShort();
  const url = await getAssetUpload(id, assetId);

  const json: PostAssetsResponse = { url, assetId };
  return NextResponse.json(json);
}
