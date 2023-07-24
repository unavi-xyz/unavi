import { NextRequest, NextResponse } from "next/server";

import { getSession } from "@/src/server/auth/getSession";
import { db } from "@/src/server/db/drizzle";
import { FixWith } from "@/src/server/db/types";

import { getWorldModelDownloadURL, getWorldModelUploadURL } from "./files";
import {
  GetFileDownloadResponse,
  GetFileUploadResponse,
  Params,
  paramsSchema,
} from "./types";

// Get file download URL
export async function GET(request: NextRequest, { params }: Params) {
  const session = await getSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the world
  const _found = await db.query.world.findFirst({
    where: (row, { eq }) =>
      eq(row.ownerId, session.user.userId) && eq(row.publicId, id),
    with: { model: true },
  });
  if (!_found) return new Response("world not found", { status: 404 });
  const found: FixWith<typeof _found, "model"> = _found;
  if (!found.model) return new Response("world not found", { status: 404 });

  const url = await getWorldModelDownloadURL(found.model.key, file);

  const json: GetFileDownloadResponse = { url };
  return NextResponse.json(json);
}

// Get file upload URL
export async function PUT(request: NextRequest, { params }: Params) {
  const session = await getSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the world
  const _found = await db.query.world.findFirst({
    where: (row, { eq }) =>
      eq(row.ownerId, session.user.userId) && eq(row.publicId, id),
    with: { model: true },
  });
  if (!_found) return new Response("world not found", { status: 404 });
  const found: FixWith<typeof _found, "model"> = _found;
  if (!found.model) return new Response("world not found", { status: 404 });

  const url = await getWorldModelUploadURL(found.model.key, file);

  const json: GetFileUploadResponse = { url };
  return NextResponse.json(json);
}
