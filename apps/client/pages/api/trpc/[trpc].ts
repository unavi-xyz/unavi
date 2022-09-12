import * as trpc from "@trpc/server";
import * as trpcNext from "@trpc/server/adapters/next";
import {
  DeleteObjectCommand,
  DeleteObjectsCommand,
  GetObjectCommand,
  PutObjectCommand,
  S3Client,
} from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";
import { MiddlewareResult } from "@trpc/server/dist/declarations/src/internals/middlewares";
import { z } from "zod";

import { Scene } from "@wired-labs/engine";

import {
  IAuthenticatedContext,
  IContext,
  createContext,
} from "../../../src/auth/context";
import { prisma } from "../../../src/auth/prisma";

const s3Client = new S3Client({
  endpoint: `https://${process.env.S3_ENDPOINT}` ?? "",
  region: process.env.S3_REGION ?? "",
  credentials: {
    accessKeyId: process.env.S3_ACCESS_KEY_ID ?? "",
    secretAccessKey: process.env.S3_SECRET ?? "",
  },
});

async function uploadScene(scene: any, id: string) {
  const data = await s3Client.send(
    new PutObjectCommand({
      Bucket: process.env.S3_BUCKET,
      Key: `${id}.json`,
      Body: JSON.stringify(scene),
      ACL: "private",
    })
  );
  return data;
}

async function uploadImage(image: any, id: string) {
  const data = await s3Client.send(
    new PutObjectCommand({
      Bucket: process.env.S3_BUCKET,
      Key: `${id}.jpeg`,
      Body: image,
      ACL: "private",
    })
  );
  return data;
}

async function getScene(id: string) {
  try {
    const command = new GetObjectCommand({
      Bucket: process.env.S3_BUCKET,
      Key: `${id}.json`,
    });
    const url = await getSignedUrl(s3Client, command, { expiresIn: 3600 });
    return url;
  } catch (e) {
    return null;
  }
}

async function getImage(id: string) {
  try {
    const command = new GetObjectCommand({
      Bucket: process.env.S3_BUCKET,
      Key: `${id}.jpeg`,
    });
    const url = await getSignedUrl(s3Client, command, { expiresIn: 3600 });
    return url;
  } catch (e) {
    return null;
  }
}

async function deleteFromS3(id: string) {
  try {
    await s3Client.send(
      new DeleteObjectsCommand({
        Bucket: process.env.S3_BUCKET,
        Delete: {
          Objects: [{ Key: `${id}.json` }, { Key: `${id}.jpeg` }],
        },
      })
    );
  } catch (e) {
    console.error(e);
  }
}

export const appRouter = trpc
  .router<IContext>()
  .query("ping", {
    resolve() {
      return "pong";
    },
  })
  .middleware(async ({ ctx: { authenticated }, next }) => {
    if (!authenticated) throw new trpc.TRPCError({ code: "UNAUTHORIZED" });
    return next() as Promise<MiddlewareResult<IAuthenticatedContext>>;
  })
  .query("projects", {
    async resolve({ ctx: { address } }) {
      const projects = await prisma.project.findMany({
        where: { owner: address },
        orderBy: { updatedAt: "desc" },
      });

      const images = await Promise.all(
        projects.map(async (project) => await getImage(project.id))
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
      const imagePromise = getImage(id);

      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new trpc.TRPCError({ code: "NOT_FOUND" });

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
      if (!project) throw new trpc.TRPCError({ code: "NOT_FOUND" });

      const url = await getScene(id);
      if (!url) return null;

      const res = await fetch(url);
      const scene: Scene = await res.json();
      return scene;
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

      // Upload image to S3
      if (image) {
        const base64str = image.split("base64,")[1]; // Remove the image type metadata.
        const imageFile = Buffer.from(base64str, "base64");

        // Limit image size to 500kb
        if (imageFile.length < 500000) {
          await uploadImage(imageFile, id);
        }
      }

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
    }),
    async resolve({
      ctx: { address },
      input: { id, name, description, image, editorState, scene },
    }) {
      // Verify that the user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new trpc.TRPCError({ code: "UNAUTHORIZED" });

      // Upload scene to S3
      await uploadScene(scene, id);

      // Upload image to S3
      if (image) {
        const base64str = image.split("base64,")[1]; // Remove the image type metadata.
        const imageFile = Buffer.from(base64str, "base64");

        // Limit image size to 500kb
        if (imageFile.length < 500000) {
          await uploadImage(imageFile, id);
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
      if (!project) throw new trpc.TRPCError({ code: "NOT_FOUND" });

      // Delete from database
      const prismaPromise = prisma.project.delete({
        where: { id },
      });

      // Delete from S3
      const s3Promise = deleteFromS3(id);

      await Promise.all([prismaPromise, s3Promise]);
    },
  });

// export type definition of API
export type AppRouter = typeof appRouter;

// export API handler
export default trpcNext.createNextApiHandler({
  router: appRouter,
  createContext,
});

// export next config
export const config = {
  api: {
    bodyParser: {
      sizeLimit: "100mb",
    },
  },
};
