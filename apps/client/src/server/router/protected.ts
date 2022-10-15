import { TRPCError } from "@trpc/server";
import { z } from "zod";

import { emptyScene } from "../../editor/constants";
import { prisma } from "../prisma";
import {
  createFileUploadURL,
  createImageUploadURL,
  createPublishedImageUploadURL,
  createPublishedMetadataUploadURL,
  createPublishedModelUploadURL,
  createSceneUploadURL,
  deleteProjectFromS3,
  getFileURL,
  getImageURL,
  getSceneURL,
} from "../s3";
import { createProtectedRouter } from "./context";

const UUID_LENGTH = 36;
const NANOID_LENGTH = 21;
const PROJECT_NAME_LENGTH = 70;
const PROJECT_DESCRIPTION_LENGTH = 2000;

export const protectedRouter = createProtectedRouter()
  .query("projects", {
    async resolve({ ctx: { address } }) {
      const projects = await prisma.project.findMany({
        where: { owner: address },
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
    },
  })

  .query("project", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      return project;
    },
  })

  .query("project-scene", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get scene url from S3
      const url = await getSceneURL(id);

      return url;
    },
  })

  .query("project-image", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get image url from S3
      const url = await getImageURL(id);

      return url;
    },
  })

  .query("project-files", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get file urls from S3
      const urlPromises = project.files.map((file) =>
        getFileURL(id, file.storageKey)
      );

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
    },
  })

  .mutation("project-scene-upload", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get scene upload URL from S3
      const url = await createSceneUploadURL(id);

      return url;
    },
  })

  .mutation("project-image-upload", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get image upload URL from S3
      const url = await createImageUploadURL(id);

      return url;
    },
  })

  .mutation("project-file-upload", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
      fileId: z.string().length(NANOID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id, fileId } }) {
      // Verify user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Add file to database if it doesn't exist
      if (!project.files.find((file) => file.storageKey === fileId)) {
        await prisma.file.create({
          data: {
            storageKey: fileId,
            projectId: id,
          },
        });
      }

      // Get file upload URL from S3
      const url = await createFileUploadURL(id, fileId);

      return url;
    },
  })

  .mutation("create-project", {
    input: z.object({
      name: z.string().max(PROJECT_NAME_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { name } }) {
      // Create project
      const { id } = await prisma.project.create({
        data: {
          owner: address,
          name,
        },
      });

      // Upload default scene to S3
      const url = await createSceneUploadURL(id);
      await fetch(url, {
        method: "PUT",
        body: JSON.stringify(emptyScene),
        headers: {
          "Content-Type": "application/json",
        },
      });

      return id;
    },
  })

  .mutation("save-project", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
      name: z.string().max(PROJECT_NAME_LENGTH).optional(),
      description: z.string().max(PROJECT_DESCRIPTION_LENGTH).optional(),
      editorState: z
        .object({
          colliders: z.boolean(),
        })
        .optional(),
    }),
    async resolve({
      ctx: { address },
      input: { id, name, description, editorState },
    }) {
      // Verify that user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "UNAUTHORIZED" });

      // Update database
      await prisma.project.update({
        where: { id },
        data: {
          name,
          description,
          editorState: JSON.stringify(editorState),
        },
      });
    },
  })

  .mutation("delete-project", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify that user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      const promises: Promise<any>[] = [];

      // Delete project from database
      promises.push(
        prisma.project.delete({
          where: { id },
          include: { files: true },
        })
      );

      // Delete project from S3
      const fileIds = project.files.map((file) => file.storageKey);
      promises.push(deleteProjectFromS3(id, fileIds));

      await Promise.all(promises);
    },
  })

  .mutation("published-model-upload", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify that user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get model upload URL from S3
      const url = await createPublishedModelUploadURL(id);

      return url;
    },
  })

  .mutation("published-image-upload", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify that user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get image upload URL from S3
      const url = await createPublishedImageUploadURL(id);

      return url;
    },
  })

  .mutation("published-metadata-upload", {
    input: z.object({
      id: z.string().length(UUID_LENGTH),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify that user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get metadata upload URL from S3
      const url = await createPublishedMetadataUploadURL(id);

      return url;
    },
  });
