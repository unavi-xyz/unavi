import { NextRequest, NextResponse } from "next/server";

import { fetchPlayerCount } from "@/src/server/helpers/fetchPlayerCount";

import { Params, paramsSchema } from "../types";

// Get player count for a space
export async function GET(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);
  const spaceId = parseInt(id);

  const playerCount = await fetchPlayerCount(spaceId);

  return NextResponse.json(playerCount);
}
