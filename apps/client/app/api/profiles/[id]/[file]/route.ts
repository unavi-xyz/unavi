import { NextRequest, NextResponse } from "next/server";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { fetchProfileOwner } from "@/src/server/helpers/fetchProfileOwner";

import { getUpload } from "../../files";
import { GetFileUploadResponse, Params } from "./types";
import { paramsSchema } from "./types";

// Get file upload URL
export async function PUT(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session || !session.user.address) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);
  const profileId = parseInt(id);

  // Verify user owns the profile
  const owner = await fetchProfileOwner(profileId);
  if (owner !== session.user.address) return new Response("Unauthorized", { status: 401 });

  const url = await getUpload(profileId, file);

  const json: GetFileUploadResponse = { url };
  return NextResponse.json(json);
}
