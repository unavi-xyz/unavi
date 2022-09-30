import { TRPCError } from "@trpc/server";
import { SceneJSON } from "@wired-labs/engine";
import { z } from "zod";

import { emptyScene } from "../../editor/constants";
import { prisma } from "../prisma";
import {
  deleteFileBlobFromS3,
  deleteProjectFromS3,
  getFileBlobFromS3,
  getImageFromS3,
  getSceneFromS3,
  uploadFileBlobToS3 as uploadFileBlobToS3,
  uploadImageToS3,
  uploadSceneToS3,
} from "../s3";
import { createProtectedRouter } from "./context";

export const protectedRouter = createProtectedRouter()
  .query("projects", {
    async resolve({ ctx: { address } }) {
      const projects = await prisma.project.findMany({
        where: { owner: address },
        orderBy: { updatedAt: "desc" },
      });

      const images = await Promise.all(
        projects.map(async (project) => await getImageFromS3(project.id))
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
      id: z.string().length(36),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      const imagePromise = getImageFromS3(id);

      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      const image = await imagePromise;

      return {
        ...project,
        image,
      };
    },
  })
  .query("scene", {
    input: z.object({
      id: z.string().length(36),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify the user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      // Get the scene from S3
      const url = await getSceneFromS3(id);
      if (!url) return null;

      const res = await fetch(url);
      if (!res.ok) throw new TRPCError({ code: "NOT_FOUND" });

      const scene: SceneJSON = await res.json();

      // Get files from S3
      const uris: {
        id: string;
        uri: string;
      }[] = [];

      scene.entities.forEach((entity) => {
        if (entity.mesh?.type === "glTF") {
          const uri = entity.mesh.uri;
          if (uri)
            uris.push({
              id: entity.id,
              uri,
            });
        }
      });

      const files = await Promise.all(
        uris.map(async (uri) => {
          const url = await getFileBlobFromS3(uri.id, id);
          const res = await fetch(url);
          if (!res.ok) throw new TRPCError({ code: "NOT_FOUND" });
          const text = await res.text();
          return { id: uri.id, text };
        })
      );

      return { scene, files };
    },
  })
  .mutation("create-project", {
    input: z.object({
      name: z.string().max(255),
      description: z.string().max(2040),
      image: z.string().optional(),
    }),
    async resolve({ ctx: { address }, input: { name, description, image } }) {
      const date = new Date();

      // Create project
      const { id } = await prisma.project.create({
        data: {
          createdAt: date,
          updatedAt: date,
          owner: address,
          name,
          description,
          editorState: null,
        },
      });

      // Upload scene to S3
      const scenePromise = uploadSceneToS3(emptyScene, id);

      // Upload image to S3
      if (image) {
        const base64str = image.split("base64,")[1]; // Remove the image type metadata.
        if (!base64str) throw new TRPCError({ code: "BAD_REQUEST" });
        const imageFile = Buffer.from(base64str, "base64");

        // Limit image size to 10MB
        if (imageFile.length < 10000000) {
          await uploadImageToS3(imageFile, id);
        }
      }

      await scenePromise;

      return id;
    },
  })
  .mutation("save-project", {
    input: z.object({
      id: z.string().length(36),
      name: z.string().max(255).optional(),
      description: z.string().max(2040).optional(),
      image: z.string().optional(),
      editorState: z.string().optional(),
      scene: z.any(),
      files: z.array(
        z.object({
          id: z.string(),
          text: z.any(),
        })
      ),
    }),
    async resolve({
      ctx: { address },
      input: { id, name, description, image, editorState, scene, files },
    }) {
      // Verify that the user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
        include: { files: true },
      });
      if (!project) throw new TRPCError({ code: "UNAUTHORIZED" });

      const promises: Promise<any>[] = [];
      const timestamp = new Date();

      const previousStorageKeys = project.files.map((file) => file.storageKey);
      const currentStorageKeys = files.map((file) => file.id);
      const unusedStorageKeys = previousStorageKeys.filter(
        (key) => !currentStorageKeys.includes(key)
      );
      const newStorageKeys = currentStorageKeys.filter(
        (key) => !previousStorageKeys.includes(key)
      );

      // Remove unused files from S3
      unusedStorageKeys.forEach((key) =>
        promises.push(deleteFileBlobFromS3(key, id))
      );

      // Remove unused files from database
      promises.push(
        prisma.file.deleteMany({
          where: {
            storageKey: {
              in: unusedStorageKeys,
            },
          },
        })
      );

      // Add new files to database
      const newFiles = newStorageKeys.map((storageKey) => ({
        storageKey,
        projectId: id,
        createdAt: timestamp,
        updatedAt: timestamp,
      }));

      promises.push(
        prisma.file.createMany({
          data: newFiles,
        })
      );

      // Upload file blobs to S3
      files.forEach((file) =>
        promises.push(uploadFileBlobToS3(file.text, file.id, id))
      );

      // Upload scene to S3
      promises.push(uploadSceneToS3(scene, id));

      // Upload image to S3
      if (image) {
        const base64str = image.split("base64,")[1]; // Remove the image type metadata.
        if (!base64str) throw new TRPCError({ code: "BAD_REQUEST" });
        const imageFile = Buffer.from(base64str, "base64");

        // Limit image size to 5MB
        if (imageFile.length < 5000000) {
          promises.push(uploadImageToS3(imageFile, id));
        }
      }

      // Save to database
      promises.push(
        prisma.project.update({
          where: { id },
          data: {
            name,
            description,
            editorState,
            updatedAt: timestamp,
          },
        })
      );

      await Promise.all(promises);

      return {
        success: true,
      };
    },
  })
  .mutation("delete-project", {
    input: z.object({
      id: z.string().length(36),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      // Verify that the user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new TRPCError({ code: "NOT_FOUND" });

      const promises: Promise<any>[] = [];

      // Delete project from database
      promises.push(
        prisma.project.delete({
          where: { id },
        })
      );

      // Delete files from S3
      const files = await prisma.file.findMany({ where: { projectId: id } });
      files.forEach((file) =>
        promises.push(deleteFileBlobFromS3(file.storageKey, id))
      );

      // Delete files from database
      promises.push(
        prisma.file.deleteMany({
          where: { projectId: id },
        })
      );

      // Delete project from S3
      promises.push(deleteProjectFromS3(id));

      await Promise.all(promises);
    },
  });
