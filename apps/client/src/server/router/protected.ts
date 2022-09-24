import { TRPCError } from "@trpc/server";
import { SceneJSON } from "@wired-labs/engine";
import { z } from "zod";

import { emptyScene } from "../../editor/constants";
import { prisma } from "../prisma";
import {
  deleteProjectFromS3,
  getImageFromS3,
  getSceneFromS3,
  uploadImageToS3,
  uploadSceneToS3,
} from "../s3";
import { createProtectedRouter } from "./context";

export const protectedRouter = createProtectedRouter()
  .query("projects", {
    async resolve({ ctx: { address } }) {
      try {
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
      } catch (e) {
        throw e;
      }
    },
  })
  .query("project", {
    input: z.object({
      id: z.string().length(36),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      try {
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
      } catch (e) {
        throw e;
      }
    },
  })
  .query("scene", {
    input: z.object({
      id: z.string().length(36),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      try {
        // Verify the user owns the project
        const project = await prisma.project.findFirst({
          where: { id, owner: address },
        });
        if (!project) throw new TRPCError({ code: "NOT_FOUND" });

        const url = await getSceneFromS3(id);
        if (!url) return null;

        const res = await fetch(url);
        if (!res.ok) throw new TRPCError({ code: "NOT_FOUND" });

        const scene: SceneJSON = await res.json();
        return scene;
      } catch (e) {
        throw e;
      }
    },
  })
  .mutation("create-project", {
    input: z.object({
      name: z.string().max(255),
      description: z.string().max(2040),
      image: z.string().optional(),
    }),
    async resolve({ ctx: { address }, input: { name, description, image } }) {
      try {
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
          const imageFile = Buffer.from(base64str, "base64");

          // Limit image size to 10MB
          if (imageFile.length < 10000000) {
            await uploadImageToS3(imageFile, id);
          }
        }

        await scenePromise;

        return id;
      } catch (e) {
        throw e;
      }
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
    }),
    async resolve({
      ctx: { address },
      input: { id, name, description, image, editorState, scene },
    }) {
      try {
        // Verify that the user owns the project
        const project = await prisma.project.findFirst({
          where: { id, owner: address },
        });
        if (!project) throw new TRPCError({ code: "UNAUTHORIZED" });

        // Upload scene to S3
        await uploadSceneToS3(scene, id);

        // Upload image to S3
        if (image) {
          const base64str = image.split("base64,")[1]; // Remove the image type metadata.
          const imageFile = Buffer.from(base64str, "base64");

          // Limit image size to 500kb
          if (imageFile.length < 500000) {
            await uploadImageToS3(imageFile, id);
          }
        }

        // Save to database
        await prisma.project.update({
          where: { id },
          data: {
            name,
            description,
            editorState,
            updatedAt: new Date(),
          },
        });

        return {
          success: true,
        };
      } catch (e) {
        throw e;
      }
    },
  })
  .mutation("delete-project", {
    input: z.object({
      id: z.string().length(36),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      try {
        // Verify that the user owns the project
        const project = await prisma.project.findFirst({
          where: { id, owner: address },
        });
        if (!project) throw new TRPCError({ code: "NOT_FOUND" });

        // Delete from database
        const prismaPromise = prisma.project.delete({
          where: { id },
        });

        // Delete from S3
        const s3Promise = deleteProjectFromS3(id);

        await Promise.all([prismaPromise, s3Promise]);
      } catch (e) {
        throw e;
      }
    },
  });
