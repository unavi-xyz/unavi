import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { nanoidShort } from "@/src/server/nanoid";
import { prisma } from "@/src/server/prisma";

import { Params, paramsSchema } from "../types";
import { PublishProjectResponse } from "./types";

// Publish project
export async function POST(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);

  // Verify user is authenticated
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { publicId: id, owner: session.address },
    include: { Space: true },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  let spaceId = found.Space?.publicId;

  // Create a new space if there is not one already
  if (!spaceId) {
    const publicId = nanoidShort();

    const newSpace = await prisma.space.create({
      data: { publicId, owner: session.address },
      select: { id: true },
    });

    // Link project to space
    await prisma.project.update({ where: { publicId: id }, data: { spaceId: newSpace.id } });

    spaceId = publicId;
  }

  const json: PublishProjectResponse = { spaceId };
  return NextResponse.json(json);
}
