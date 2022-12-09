import { TRPCError } from "@trpc/server";
import { z } from "zod";

import {
  createPublishedImageUploadURL,
  createPublishedMetadataUploadURL,
  createPublishedModelUploadURL,
  deletePublicationFromS3,
} from "../s3";
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

  create: protectedProcedure
    .input(
      z.object({
        type: z.enum(["SPACE", "AVATAR"]),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Create publication
      const { id } = await ctx.prisma.publication.create({
        data: { owner: ctx.session.address, type: input.type },
      });

      return id;
    }),

  link: protectedProcedure
    .input(
      z.object({
        lensId: z.string(),
        publicationId: z.string().length(PUBLICATION_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the publication
      const publication = await ctx.prisma.publication.findFirst({
        where: { id: input.publicationId, owner: ctx.session.address },
      });
      if (!publication) throw new TRPCError({ code: "NOT_FOUND" });

      // Save lensId to publication
      await ctx.prisma.publication.update({
        where: { id: input.publicationId },
        data: { lensId: input.lensId },
      });
    }),

  delete: protectedProcedure
    .input(
      z.object({
        lensId: z.string().optional(),
        publicationId: z.string().length(PUBLICATION_ID_LENGTH).optional(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      let id = input.publicationId;

      // If lensId is provided, find the publication
      if (input.lensId) {
        const publication = await ctx.prisma.publication.findFirst({
          where: { lensId: input.lensId, owner: ctx.session.address },
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
