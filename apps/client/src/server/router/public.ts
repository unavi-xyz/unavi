import { TRPCError } from "@trpc/server";
import { nanoid } from "nanoid";
import { z } from "zod";

import { env } from "../../env/client.mjs";
import { getModelStats } from "../helpers/getModelStats";
import { getSpaceMetadata } from "../helpers/getSpaceMetadata";
import { createTempFileUploadURL } from "../s3";
import { publicProcedure, router } from "./trpc";

const HOST_URL =
  process.env.NODE_ENV === "development"
    ? "http://localhost:4000"
    : `https://${env.NEXT_PUBLIC_DEFAULT_HOST}`;

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
        id: z.number(),
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
        spaceId: z.number().int(),
      })
    )
    .mutation(async ({ input, ctx }) => {
      const promises: Promise<any>[] = [];

      // Verify space exists
      const space = await getSpaceMetadata(input.spaceId);
      if (!space) throw new TRPCError({ code: "NOT_FOUND" });

      // Get publication id
      const publication = await ctx.prisma.publication.findFirst({
        where: { spaceId: input.spaceId },
      });

      let publicationId: string;

      if (publication) {
        publicationId = publication.id;
      } else {
        // If no publication, create one
        const { id } = await ctx.prisma.publication.create({
          data: { spaceId: input.spaceId },
        });

        publicationId = id;
      }

      // Create space view event
      promises.push(ctx.prisma.viewEvent.create({ data: { publicationId } }));

      // Update space view count
      promises.push(
        ctx.prisma.publication.update({
          where: { id: publicationId },
          data: { viewCount: { increment: 1 } },
        })
      );

      await Promise.all(promises);
    }),

  modelStats: publicProcedure
    .input(
      z.object({
        publicationId: z.string(),
      })
    )
    .query(async ({ input }) => {
      const stats = await getModelStats(input.publicationId);
      return stats;
    }),
});
