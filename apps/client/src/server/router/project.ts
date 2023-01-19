import { TRPCError } from "@trpc/server";
import { customAlphabet } from "nanoid";
import { z } from "zod";

import { Project } from "../s3/Project";
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
        publicationId: z.string().length(PUBLICATION_ID_LENGTH).optional(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
      const found = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      // Verify user owns the publication
      if (input.publicationId !== undefined) {
        const found = await ctx.prisma.publication.findFirst({
          where: { id: input.publicationId, owner: ctx.session.address },
        });
        if (!found) throw new TRPCError({ code: "BAD_REQUEST" });
      }

      await ctx.prisma.project.update({
        where: { id: input.id },
        data: {
          name: input.name,
          description: input.description,
          publicationId: input.publicationId,
        },
      });
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
      });
      if (!found) throw new TRPCError({ code: "NOT_FOUND" });

      const promises: Promise<any>[] = [];

      // Delete from database
      promises.push(ctx.prisma.project.delete({ where: { id: input.id } }));

      // Delete from S3
      const project = new Project(input.id);
      promises.push(project.delete());

      await Promise.all(promises);
    }),
});
