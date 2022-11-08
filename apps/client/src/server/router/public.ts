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
    // Get publications
    const publications = await prisma.publication.findMany({
      where: { type: "SPACE" },
      orderBy: { viewCount: "desc" },
      take: 9,
    });

    // Get lens ids
    const lensIds: string[] = [];

    publications.forEach((publication) => {
      if (publication.lensId) lensIds.push(publication.lensId);
    });

    return lensIds;
  }),

  hotAvatars: publicProcedure.query(async () => {
    // Get publications
    const avatars = await prisma.publication.findMany({
      where: { type: "SPACE" },
      orderBy: { viewCount: "desc" },
      take: 9,
    });

    // Get lens ids
    const lensIds: string[] = [];

    avatars.forEach((publication) => {
      if (publication.lensId) lensIds.push(publication.lensId);
    });

    return lensIds;
  }),

  addView: publicProcedure
    .input(
      z.object({
        lensId: z.string(),
      })
    )
    .mutation(async ({ input }) => {
      const promises: Promise<any>[] = [];

      // Get publication id
      const publication = await prisma.publication.findFirst({
        where: { lensId: input.lensId },
      });
      if (!publication) return;

      // Create space view event
      promises.push(
        prisma.viewEvent.create({ data: { publicationId: publication.id } })
      );

      // Update space view count
      promises.push(
        prisma.publication.update({
          where: { id: publication.id },
          data: { viewCount: { increment: 1 } },
        })
      );
    }),
});
