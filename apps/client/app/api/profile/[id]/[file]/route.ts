import { NextResponse } from "next/server";

import { fetchProfileOwner } from "../../../../../src/server/helpers/fetchProfileOwner";
import { getServerSession } from "../../../../../src/server/helpers/getServerSession";
import { getDownload } from "../../files";
import { Params } from "./types";
import { GetFileDownloadResponse, paramsSchema } from "./types";

// Get file download URL
export async function GET(request: Request, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id, file } = paramsSchema.parse(params);
  const profileId = parseInt(id);

  // Verify user owns the profile
  const owner = await fetchProfileOwner(profileId);
  if (owner !== session.address) return new Response("Unauthorized", { status: 401 });

  const url = await getDownload(profileId, file);

  const json: GetFileDownloadResponse = { url };
  return NextResponse.json(json);
}
