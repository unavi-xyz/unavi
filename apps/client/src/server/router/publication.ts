import { TRPCError } from "@trpc/server";
import { z } from "zod";

import { protectedProcedure, router } from "./trpc";

const PUBLICATION_ID_LENGTH = 25; // cuid

export const publicationRouter = router({
  modelUploadURL: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PUBLICATION_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the publication
      const publication = await ctx.prisma.publication.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!publication) throw new TRPCError({ code: "NOT_FOUND" });

      // Get model upload URL from S3
      const url = await createPublishedModelUploadURL(input.id);

      return url;
    }),

  imageUploadURL: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PUBLICATION_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the publication
      const publication = await ctx.prisma.publication.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!publication) throw new TRPCError({ code: "NOT_FOUND" });

      // Get image upload URL from S3
      const url = await createPublishedImageUploadURL(input.id);

      return url;
    }),

  metadataUploadURL: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PUBLICATION_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the publication
      const publication = await ctx.prisma.publication.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!publication) throw new TRPCError({ code: "NOT_FOUND" });

      // Get metadata upload URL from S3
      const url = await createPublishedMetadataUploadURL(input.id);

      return url;
    }),

  create: protectedProcedure.mutation(async ({ ctx }) => {
    // Create publication
    const { id } = await ctx.prisma.publication.create({
      data: { owner: ctx.session.address },
    });

    return id;
  }),

  link: protectedProcedure
    .input(
      z.object({
        spaceId: z.number().int(),
        publicationId: z.string().length(PUBLICATION_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the publication
      const publication = await ctx.prisma.publication.findFirst({
        where: { id: input.publicationId, owner: ctx.session.address },
      });
      if (!publication) throw new TRPCError({ code: "NOT_FOUND" });

      // Save spaceId to publication
      await ctx.prisma.publication.update({
        where: { id: input.publicationId },
        data: { spaceId: input.spaceId },
      });
    }),

  delete: protectedProcedure
    .input(
      z.object({
        spaceId: z.number().int().optional(),
        publicationId: z.string().length(PUBLICATION_ID_LENGTH).optional(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      let id = input.publicationId;

      // If spaceId is provided, find the publication
      if (input.spaceId) {
        const publication = await ctx.prisma.publication.findFirst({
          where: { spaceId: input.spaceId, owner: ctx.session.address },
        });
        if (publication) id = publication.id;
      }

      if (!id) throw new TRPCError({ code: "BAD_REQUEST" });

      // Verify user owns the publication
      const publication = await ctx.prisma.publication.findFirst({
        where: { id, owner: ctx.session.address },
      });
      if (!publication) throw new TRPCError({ code: "NOT_FOUND" });

      const promises: Promise<any>[] = [];

      // Delete publication from S3
      promises.push(deletePublicationFromS3(id));

      // Remove publicationId from projects
      await ctx.prisma.project.updateMany({
        where: { publicationId: id },
        data: { publicationId: null },
      });

      // Delete publication from database
      await ctx.prisma.publication.delete({
        where: { id },
        include: { ViewEvents: true },
      });

      await Promise.all(promises);
    }),
});
