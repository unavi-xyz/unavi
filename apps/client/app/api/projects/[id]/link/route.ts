import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { prisma } from "@/src/server/prisma";

import { Params, paramsSchema } from "../../../spaces/[id]/types";
import { putSchema } from "./types";

// Link project to space
export async function PUT(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);
  const { spaceId } = putSchema.parse(await request.json());

  // Verify user owns the project
  const found = await prisma.project.findFirst({ where: { publicId: id, owner: session.address } });
  if (!found) return new Response("Project not found", { status: 404 });

  // Verify user owns the space
  const space = await prisma.space.findFirst({
    where: { publicId: spaceId, owner: session.address },
  });
  if (!space) return new Response("Space not found", { status: 404 });

  // Link project to space
  await prisma.project.update({ where: { publicId: id }, data: { spaceId: space.id } });

  return NextResponse.json({ success: true });
}
