import { nanoid } from "nanoid";

import { prisma } from "../prisma";
import { createTempFileUploadURL } from "../s3";
import { publicProcedure, router } from "./trpc";

export const publicRouter = router({
  tempUploadURL: publicProcedure.mutation(async () => {
    // Get temp file upload URL from S3
    const fileId = nanoid();
    const url = await createTempFileUploadURL(fileId);
    return { url, fileId };
  }),

  hotSpaces: publicProcedure.query(async () => {
    const spaces = await prisma.space.findMany({
      orderBy: { viewsCount: "desc" },
      take: 10,
    });

    const spaceIds = spaces.map((space) => space.publicationId);
    return spaceIds;
  }),

  hotAvatars: publicProcedure.query(async () => {
    const avatars = await prisma.avatar.findMany({
      orderBy: { viewsCount: "desc" },
      take: 10,
    });

    const avatarIds = avatars.map((avatar) => avatar.publicationId);
    return avatarIds;
  }),
});
