import { NextRequest, NextResponse } from "next/server";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { nanoidShort } from "@/src/server/nanoid";
import { prisma } from "@/src/server/prisma";

import { Params, paramsSchema } from "../types";
import { PostMintResponse } from "./types";

// Create new space NFT
export async function POST(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({
    include: { SpaceModel: true, SpaceNFT: true },
    where: { ownerId: session.user.userId, publicId: id },
  });
  if (!found) return new Response("Space not found", { status: 404 });

  let nftId = found.SpaceNFT?.publicId;

  // Create NFT if it doesn't exist
  if (!nftId) {
    const publicId = nanoidShort();

    await prisma.spaceNFT.create({ data: { publicId, spaceId: found.id } });

    nftId = publicId;
  }

  const json: PostMintResponse = { nftId };
  return NextResponse.json(json);
}
