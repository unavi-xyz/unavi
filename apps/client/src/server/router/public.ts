import { nanoid } from "nanoid";
import { z } from "zod";

import { prisma } from "../prisma";
import { createTempFileUploadURL } from "../s3";
import { publicProcedure, router } from "./trpc";

const HOST_URL =
  process.env.NODE_ENV === "development"
    ? "http://localhost:4000"
    : "https://host.thewired.space";

export const publicRouter = router({
  tempUploadURL: publicProcedure.mutation(async () => {
    // Get temp file upload URL from S3
    const fileId = nanoid();
    const url = await createTempFileUploadURL(fileId);
    return { url, fileId };
  }),

  playerCount: publicProcedure
    .input(
      z.object({
        id: z.string(),
      })
    )
    .query(async ({ input }) => {
      const response = await fetch(`${HOST_URL}/playercount/${input.id}`);
      const playerCountText = await response.text();
      const playerCount = parseInt(playerCountText);

      return playerCount;
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

      let publicationId: string;

      if (publication) {
        publicationId = publication.id;
      } else {
        // If no publication, create one
        const { id } = await prisma.publication.create({
          data: { type: "SPACE", lensId: input.lensId },
        });

        publicationId = id;
      }

      // Create space view event
      promises.push(prisma.viewEvent.create({ data: { publicationId } }));

      // Update space view count
      promises.push(
        prisma.publication.update({
          where: { id: publicationId },
          data: { viewCount: { increment: 1 } },
        })
      );

      await Promise.all(promises);
    }),
});
