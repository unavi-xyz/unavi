import { cache } from "react";

import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { prisma } from "../prisma";

export const fetchDBSpaceURI = cache(async (id: string) => {
  try {
    const space = await prisma.space.findFirst({
      where: { publicId: id },
      select: { SpaceModel: { select: { publicId: true } }, tokenId: true },
    });
    if (!space || !space.SpaceModel) throw new Error("Space not found");

    const uri = cdnURL(S3Path.spaceModel(space.SpaceModel.publicId).metadata);

    return { uri, tokenId: space.tokenId };
  } catch {
    return null;
  }
});
