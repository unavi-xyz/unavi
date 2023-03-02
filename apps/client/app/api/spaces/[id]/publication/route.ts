import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "../../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../../src/server/prisma";
import { Params, paramsSchema } from "../types";

// Get publication
export async function GET(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);
  const spaceId = parseInt(id);

  // Verify user owns the publication
  const publication = await prisma.publication.findFirst({
    where: { spaceId, owner: session.address },
  });

  return NextResponse.json(publication);
}
