import { NextRequest, NextResponse } from "next/server";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { nanoidShort } from "@/src/server/nanoid";
import { prisma } from "@/src/server/prisma";

import { Params, paramsSchema } from "../types";
import { PublishProjectResponse } from "./types";

// Publish project
export async function POST(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);

  // Verify user is authenticated
  const session = await getUserSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    include: { Space: { include: { SpaceNFT: true } } },
    where: { ownerId: session.user.userId, publicId: id },
  });
  if (!found) return new Response("Project not found", { status: 404 });

  const nftId = found.Space?.SpaceNFT?.publicId;
  let spaceId = found.Space?.publicId;

  // Create a new space if there is not one already
  if (!spaceId) {
    const publicId = nanoidShort();

    const newSpace = await prisma.space.create({
      data: { ownerId: session.user.userId, publicId },
      select: { id: true },
    });

    // Link project to space
    await prisma.project.update({ data: { spaceId: newSpace.id }, where: { publicId: id } });

    spaceId = publicId;
  }

  const json: PublishProjectResponse = { nftId, spaceId };
  return NextResponse.json(json);
}
