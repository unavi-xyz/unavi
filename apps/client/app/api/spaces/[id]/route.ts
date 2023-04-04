import { NextRequest, NextResponse } from "next/server";

import { fetchSpace } from "@/src/server/helpers/fetchSpace";

import { Params, paramsSchema } from "./types";

// Get space
export async function GET(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);
  const spaceId = parseInt(id);

  const space = await fetchSpace(spaceId);

  return NextResponse.json(space);
}
