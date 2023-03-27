import { NextRequest, NextResponse } from "next/server";

import { fetchProfileFromAddress } from "../../../../../src/server/helpers/fetchProfileFromAddress";
import { Params } from "./types";

export const runtime = "edge";

// Get profile
export async function GET(request: NextRequest, { params }: Params) {
  const profile = await fetchProfileFromAddress(params.address);
  return NextResponse.json(profile);
}
