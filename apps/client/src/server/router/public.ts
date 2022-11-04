import { nanoid } from "nanoid";
import { z } from "zod";

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

  spaceView: publicProcedure
    .input(
      z.object({
        id: z.string(),
      })
    )
    .mutation(async ({ input }) => {
      // Update space view count
      await prisma.space.upsert({
        create: { publicationId: input.id, viewsCount: 1 },
        where: { publicationId: input.id },
        update: { viewsCount: { increment: 1 } },
      });

      // Create space view event
      await prisma.spaceViewEvent.create({ data: { spaceId: input.id } });
    }),

  avatarView: publicProcedure
    .input(
      z.object({
        id: z.string(),
      })
    )
    .mutation(async ({ input }) => {
      // Update avatar view count
      await prisma.avatar.upsert({
        create: { publicationId: input.id, viewsCount: 1 },
        where: { publicationId: input.id },
        update: { viewsCount: { increment: 1 } },
      });

      // Create avatar view event
      await prisma.avatarViewEvent.create({ data: { avatarId: input.id } });
    }),
});
