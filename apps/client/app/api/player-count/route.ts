import { NextRequest, NextResponse } from "next/server";

import { fetchPlayerCount } from "@/src/server/helpers/fetchPlayerCount";

import { PostPlayerCountResponse, postPlayerCountSchema } from "./types";

// Get world player count
export async function POST(request: NextRequest) {
  const { host, uri } = postPlayerCountSchema.parse(await request.json());

  const count = await fetchPlayerCount(uri, host);

  const json: PostPlayerCountResponse = { count };
  return NextResponse.json(json);
}
