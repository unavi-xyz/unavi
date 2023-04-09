import { NextRequest, NextResponse } from "next/server";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { nanoidShort } from "@/src/server/nanoid";
import { prisma } from "@/src/server/prisma";

import { Params, paramsSchema } from "../types";
import { PostMintResponse } from "./types";

// Create new space NFT
export async function POST(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the space
  const found = await prisma.space.findFirst({
    where: { publicId: id, owner: session.address },
    include: { SpaceModel: true, SpaceNFT: true },
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
