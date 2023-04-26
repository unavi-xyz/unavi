import { NextRequest, NextResponse } from "next/server";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { prisma } from "@/src/server/prisma";

import { getSpaceNFTDownloadURL, getSpaceNFTUploadURL } from "./files";
import { GetFileDownloadResponse, GetFileUploadResponse, Params, paramsSchema } from "./types";

// Get file download URL
export async function GET(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session || !session.user.address) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({
    where: { publicId: id, owner: session.user.address },
    include: { SpaceNFT: true },
  });
  if (!found?.SpaceNFT) return new Response("Space not found", { status: 404 });

  const url = await getSpaceNFTDownloadURL(found.SpaceNFT.publicId, file);

  const json: GetFileDownloadResponse = { url };
  return NextResponse.json(json);
}

// Get file upload URL
export async function PUT(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session || !session.user.address) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({
    where: { publicId: id, owner: session.user.address },
    include: { SpaceNFT: true },
  });
  if (!found?.SpaceNFT) return new Response("Space not found", { status: 404 });

  const url = await getSpaceNFTUploadURL(found.SpaceNFT.publicId, file);

  const json: GetFileUploadResponse = { url };
  return NextResponse.json(json);
}
