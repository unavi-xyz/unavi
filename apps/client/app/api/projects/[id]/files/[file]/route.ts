import { NextRequest, NextResponse } from "next/server";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { prisma } from "@/src/server/prisma";

import { Params } from "../../types";
import { getProjectDownloadURL, getProjectUploadURL } from "./files";
import { GetFileDownloadResponse, GetFileUploadResponse, paramsSchema } from "./types";

// Get file download URL
export async function GET(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { ownerId: session.user.userId, publicId: id },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  const url = await getProjectDownloadURL(id, file);

  const json: GetFileDownloadResponse = { url };
  return NextResponse.json(json);
}

// Get file upload URL
export async function PUT(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { ownerId: session.user.userId, publicId: id },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  const url = await getProjectUploadURL(id, file);

  const json: GetFileUploadResponse = { url };
  return NextResponse.json(json);
}
