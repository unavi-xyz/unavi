import { TRPCError } from "@trpc/server";
import { customAlphabet } from "nanoid";
import { z } from "zod";

import {
  createFileUploadURL,
  createImageUploadURL,
  createSceneUploadURL,
  deleteFilesFromS3,
  deleteProjectFromS3,
  getFileURL,
  getImageURL,
  getSceneURL,
} from "../s3";
import { protectedProcedure, router } from "./trpc";

const PROJECT_ID_LENGTH = 21;
const PUBLICATION_ID_LENGTH = 25; // cuid
const PROJECT_NAME_LENGTH = 70;
const PROJECT_DESCRIPTION_LENGTH = 2000;

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
      projects.map(async (project) => await getImageURL(project.id))
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
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      return project;
    }),

  scene: protectedProcedure
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

      // Get scene url from S3
      const url = await getSceneURL(input.id);

      return url;
    }),

  image: protectedProcedure
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

      // Get image url from S3
      const url = await getImageURL(input.id);

      return url;
    }),

  files: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
      })
    )
    .query(async ({ ctx, input }) => {
      // Verify user owns the project
      const project = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get file urls from S3
      const urlPromises = project.files.map((file) => getFileURL(input.id, file.storageKey));

      const urls = await Promise.all(urlPromises);

      const response = project.files.map((file, index) => {
        const uri = urls[index];
        if (!uri) throw new Error("Failed to get file url");

        return {
          id: file.storageKey,
          uri,
        };
      });

      return response;
    }),

  sceneUploadURL: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
      const project = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get scene upload URL from S3
      const url = await createSceneUploadURL(input.id);

      return url;
    }),

  imageUploadURL: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
      const project = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get image upload URL from S3
      const url = await createImageUploadURL(input.id);

      return url;
    }),

  fileUploadURL: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
        storageKey: z.string(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
      const project = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Add file to database if it doesn't exist
      if (!project.files.find((file) => file.storageKey === input.storageKey)) {
        await ctx.prisma.file.create({
          data: {
            storageKey: input.storageKey,
            projectId: input.id,
          },
        });
      }

      // Get file upload URL from S3
      const url = await createFileUploadURL(input.id, input.storageKey);
      return url;
    }),

  create: protectedProcedure
    .input(
      z.object({
        name: z.string().max(PROJECT_NAME_LENGTH),
      })
    )
    .mutation(async ({ ctx, input }) => {
      const id = nanoid();

      // Create project
      await ctx.prisma.project.create({
        data: {
          id,
          owner: ctx.session.address,
          name: input.name,
        },
      });

      return id;
    }),

  save: protectedProcedure
    .input(
      z.object({
        id: z.string().length(PROJECT_ID_LENGTH),
        name: z.string().max(PROJECT_NAME_LENGTH).optional(),
        description: z.string().max(PROJECT_DESCRIPTION_LENGTH).optional(),
        publicationId: z.string().length(PUBLICATION_ID_LENGTH).or(z.null()).optional(),
        editorState: z
          .object({
            visuals: z.boolean(),
            tool: z.string(),
          })
          .optional(),
        fileIds: z.array(z.string()).optional(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      // Verify user owns the project
      const project = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "UNAUTHORIZED" });

      const promises: Promise<any>[] = [];

      // Update project
      promises.push(
        ctx.prisma.project.update({
          where: { id: input.id },
          data: {
            name: input.name,
            description: input.description,
            editorState: JSON.stringify(input.editorState),
            publicationId: input.publicationId,
          },
        })
      );

      // Delete old files
      if (input.fileIds) {
        // From database
        promises.push(
          ctx.prisma.file.deleteMany({
            where: {
              projectId: input.id,
              storageKey: { notIn: input.fileIds },
            },
          })
        );

        // From S3
        const storageKeys = project.files.map((file) => file.storageKey);
        const oldFiles = storageKeys.filter((storageKey) => !input.fileIds?.includes(storageKey));
        promises.push(deleteFilesFromS3(input.id, oldFiles));
      }

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
      const project = await ctx.prisma.project.findFirst({
        where: { id: input.id, owner: ctx.session.address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      const promises: Promise<any>[] = [];

      // Delete files from database
      await ctx.prisma.file.deleteMany({
        where: { projectId: input.id },
      });

      // Delete project from database
      promises.push(
        ctx.prisma.project.delete({
          where: { id: input.id },
          include: { files: true },
        })
      );

      // Delete project from S3
      const storageKeys = project.files.map((file) => file.storageKey);
      promises.push(deleteProjectFromS3(input.id, storageKeys));

      await Promise.all(promises);
    }),
});
