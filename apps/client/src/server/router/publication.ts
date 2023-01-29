import { TRPCError } from "@trpc/server";
import { z } from "zod";

import { Publication } from "../s3/Publication";
import { protectedProcedure, router } from "./trpc";

const PUBLICATION_ID_LENGTH = 25; // cuid

export const publicationRouter = router({
  getModelUpload: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PUBLICATION_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the publication
      const found = await ctx.prisma.publication.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      const publication = new Publication(input.id);
      const url = await publication.getUpload("model");

      return url;
    }),

  getImageUpload: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PUBLICATION_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the publication
      const found = await ctx.prisma.publication.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      const publication = new Publication(input.id);
      const url = await publication.getUpload("image");

      return url;
    }),

  getMetadataUpload: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PUBLICATION_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the publication
      const found = await ctx.prisma.publication.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      const publication = new Publication(input.id);
      const url = await publication.getUpload("metadata");

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
        spaceId: z.number().int().positive(),
        publicationId: z.string().length(PUBLICATION_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the publication
      const found = await ctx.prisma.publication.findFirst({
        where: { id: input.publicationId, owner: ctx.session.address },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

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
      const found = await ctx.prisma.publication.findFirst({
        where: { id, owner: ctx.session.address },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      // Delete publication from S3
      const publication = new Publication(id);
      await publication.delete();

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
    }),

  bySpaceId: protectedProcedure
    .input(
      z.object({
        spaceId: z.number().int().positive(),
      })
    )
    .query(async ({ ctx, input }) => {
      const publication = await ctx.prisma.publication.findFirst({
        where: { spaceId: input.spaceId, owner: ctx.session.address },
      });

      return publication;
    }),
});
