import { NextRequest, NextResponse } from "next/server";

import { fetchUserProfileDB } from "@/src/server/helpers/fetchUserProfile";

type Params = { params: { username: string } };

/**
 * Fetches a user's profile given their username
 */
export async function GET(request: NextRequest, { params }: Params) {
  const json = await fetchUserProfileDB(params.username);
  return NextResponse.json(json);
}
