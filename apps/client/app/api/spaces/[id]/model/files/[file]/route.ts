import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { prisma } from "@/src/server/prisma";

import { getSpaceModelDownloadURL, getSpaceModelUploadURL } from "./files";
import { GetFileDownloadResponse, GetFileUploadResponse, Params, paramsSchema } from "./types";

// Get file download URL
export async function GET(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({ where: { publicId: id, owner: session.address } });
  if (!found) return new Response("Space not found", { status: 404 });

  const url = await getSpaceModelDownloadURL(id, file);

  const json: GetFileDownloadResponse = { url };
  return NextResponse.json(json);
}

// Get file upload URL
export async function PUT(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({ where: { publicId: id, owner: session.address } });
  if (!found) return new Response("Space not found", { status: 404 });

  const url = await getSpaceModelUploadURL(id, file);

  const json: GetFileUploadResponse = { url };
  return NextResponse.json(json);
}
