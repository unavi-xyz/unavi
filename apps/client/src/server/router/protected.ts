import { TRPCError } from "@trpc/server";
import { SceneJSON } from "@wired-labs/engine";
import { z } from "zod";

import { emptyScene } from "../../editor/constants";
import { prisma } from "../prisma";
import {
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
      });
      if (!project) throw new TRPCError({ code: "UNAUTHORIZED" });

      // Upload file blobs to S3
      await Promise.all(
        files.map((file) => uploadFileBlobToS3(file.text, file.id, id))
      );

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

      // Delete from database
      const prismaPromise = prisma.project.delete({
        where: { id },
      });

      // Delete from S3
      const s3Promise = deleteProjectFromS3(id);

      await Promise.all([prismaPromise, s3Promise]);
    },
  });
