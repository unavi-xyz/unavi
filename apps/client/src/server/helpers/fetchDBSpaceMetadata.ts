import { cache } from "react";

import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { prisma } from "../prisma";
import { readSpaceMetadata, SpaceMetadata } from "./readSpaceMetadata";

export const fetchDBSpaceMetadata = cache(async (id: string): Promise<DBSpaceMetadata | null> => {
  try {
    const space = await prisma.space.findFirst({
      where: { publicId: id },
      select: { SpaceModel: { select: { publicId: true } }, tokenId: true },
    });
    if (!space || !space.SpaceModel) throw new Error("Space not found");

    const modelURI = cdnURL(S3Path.spaceModel(space.SpaceModel.publicId).model);
    const metadata = await readSpaceMetadata(modelURI);
    if (!metadata) throw new Error("Invalid space metadata");

    return {
      ...metadata,
      tokenId: space.tokenId,
    };
  } catch {
    return null;
  }
});

export interface DBSpaceMetadata extends SpaceMetadata {
  tokenId: number | null;
}
