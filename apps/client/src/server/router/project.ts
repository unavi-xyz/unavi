import { PutObjectCommand } from "@aws-sdk/client-s3";
import { TRPCError } from "@trpc/server";
import { customAlphabet } from "nanoid";
import { z } from "zod";

import { env } from "../../env/server.mjs";
import { s3Client } from "../s3/client";
import { Project } from "../s3/Project";
import { Publication } from "../s3/Publication";
import { protectedProcedure, router } from "./trpc";

const PROJECT_ID_LENGTH = 21; // nanoid length
const PUBLICATION_ID_LENGTH = 25; // cuid length
const MAX_NAME_LENGTH = 80;
const MAX_DESCRIPTION_LENGTH = 500;

const nanoid = customAlphabet(
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz",
  PROJECT_ID_LENGTH
);

export const projectRouter = router({
  getAll: protectedProcedure.query(async ({ ctx }) => {
    const projects = await ctx.prisma.project.findMany({
      where: { owner: ctx.session.address },
      include: { Publication: true },
      orderBy: { updatedAt: "desc" },
    });

    const images = await Promise.all(
      projects.map(({ id }) => {
        const project = new Project(id);
        return project.getDownload("image");
      })
    );

    const response = projects.map((project, index) => ({
      ...project,
      image: images[index],
    }));

    return response;
  }),

  get: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
      })
    )
    .query(async ({ ctx, input }) => {
      // Verify user owns the project
      const project = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
        include: { Publication: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      return project;
    }),

  image: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
      })
    )
    .query(async ({ ctx, input }) => {
      // Verify user owns the project
      const found = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      const project = new Project(input.id);
      const url = await project.getDownload("image");

      return url;
    }),

  model: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
      })
    )
    .query(async ({ ctx, input }) => {
      // Verify user owns the project
      const found = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      const project = new Project(input.id);
      const url = await project.getDownload("model");

      return url;
    }),

  getImageUpload: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
      const found = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      const project = new Project(input.id);
      const url = await project.getUpload("image");

      return url;
    }),

  getModelUpload: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
      const found = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      const project = new Project(input.id);
      const url = await project.getUpload("model");

      return url;
    }),

  create: protectedProcedure
    .input(
      z.object({
        name: z.string().max(MAX_NAME_LENGTH).optional(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      const id = nanoid();

      await ctx.prisma.project.create({
        data: { id, owner: ctx.session.address, name: input.name ?? "", description: "" },
      });

      return id;
    }),

  update: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
        name: z.string().max(MAX_NAME_LENGTH).optional(),
        description: z.string().max(MAX_DESCRIPTION_LENGTH).optional(),
        publicationId: z.string().length(PUBLICATION_ID_LENGTH).nullable().optional(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
      const found = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
        include: { Publication: true },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      const promises: Promise<unknown>[] = [];

      // Verify user owns the publication
      if (input.publicationId) {
        const publication = await ctx.prisma.publication.findFirst({
          where: { id: input.publicationId, owner: ctx.session.address },
        });
        if (!publication) throw new TRPCError({ code: "BAD_REQUEST" });
      }

      // Delete old publication if not in use
      if (input.publicationId !== undefined) {
        if (found.Publication?.spaceId === null) {
          promises.push(ctx.prisma.publication.delete({ where: { id: found.Publication.id } }));
        }
      }

      promises.push(
        ctx.prisma.project.update({
          where: { id: input.id },
          data: {
            name: input.name,
            description: input.description,
            publicationId: input.publicationId,
          },
        })
      );

      await Promise.all(promises);
    }),

  delete: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
      const found = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
        include: { Publication: true },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      const project = new Project(input.id);
      const publicationInUse = Boolean(found.Publication && found.Publication.spaceId !== null);

      await Promise.all([
        // Delete from database
        ctx.prisma.project.delete({
          where: { id: input.id },
          include: { Publication: !publicationInUse },
        }),
        // Delete from S3
        project.delete(),
      ]);
    }),

  publish: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
      const found = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
        include: { Publication: true },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      // Create new publication if it doesn't exist
      let publicationId = found.Publication?.id;

      if (!publicationId) {
        const newPublication = await ctx.prisma.publication.create({
          data: { owner: ctx.session.address },
          select: { id: true },
        });

        publicationId = newPublication.id;

        // Update project
        await ctx.prisma.project.update({ where: { id: input.id }, data: { publicationId } });
      }

      // Create optimized model
      const project = new Project(input.id);
      await project.optimize();

      // Download optimized model
      const url = await project.getDownload("optimized_model");
      const response = await fetch(url);
      const buffer = await response.arrayBuffer();
      const array = new Uint8Array(buffer);

      // Upload optimized model to publication bucket
      const publication = new Publication(publicationId);
      const Key = publication.getKey("model");
      const ContentType = publication.getContentType("model");
      const command = new PutObjectCommand({
        Bucket: env.S3_BUCKET,
        Key,
        ContentType,
        Body: array,
        ACL: "public-read",
      });
      await s3Client.send(command);

      return publicationId;
    }),
});
