import { NextResponse } from "next/server";

import { fetchSpace } from "../../../../src/server/helpers/fetchSpace";
import { paramsSchema } from "./types";

type Params = { params: { id: string } };

// Get space
export async function GET(request: Request, { params }: Params) {
  const { id } = paramsSchema.parse(params);
  const spaceId = parseInt(id);

  const space = await fetchSpace(spaceId);

  return NextResponse.json(space);
}
