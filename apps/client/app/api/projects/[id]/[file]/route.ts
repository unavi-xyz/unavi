import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { prisma } from "@/src/server/prisma";

import { getProjectDownload, getProjectUpload } from "../../files";
import { Params } from "../types";
import { GetFileDownloadResponse, GetFileUploadResponse, paramsSchema } from "./types";

// Get file download URL
export async function GET(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  const url = await getProjectDownload(id, file);

  const json: GetFileDownloadResponse = { url };
  return NextResponse.json(json);
}

// Get file upload URL
export async function PUT(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  const url = await getProjectUpload(id, file);

  const json: GetFileUploadResponse = { url };
  return NextResponse.json(json);
}
