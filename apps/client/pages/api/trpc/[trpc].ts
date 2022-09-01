import * as trpc from "@trpc/server";
import * as trpcNext from "@trpc/server/adapters/next";
import {
  GetObjectCommand,
  PutObjectCommand,
  S3Client,
} from "@aws-sdk/client-s3";
import { MiddlewareResult } from "@trpc/server/dist/declarations/src/internals/middlewares";
import { z } from "zod";

import {
  IAuthenticatedContext,
  IContext,
  createContext,
} from "../../../src/login/context";
import { prisma } from "../../../src/login/prisma";

const BUCKET_NAME = "wired";

const s3Client = new S3Client({
  endpoint: "https://nyc3.digitaloceanspaces.com",
  region: "nyc3",
  credentials: {
    accessKeyId: process.env.S3_ACCESS_KEY_ID ?? "",
    secretAccessKey: process.env.S3_SECRET ?? "",
  },
});

async function uploadWorld(world: any, id: number) {
  const data = await s3Client.send(
    new PutObjectCommand({
      Bucket: BUCKET_NAME,
      Key: `${id}.json`,
      Body: JSON.stringify(world),
      ACL: "private",
    })
  );
  return data;
}

async function getWorld(id: number) {
  try {
    const { Body } = await s3Client.send(
      new GetObjectCommand({
        Bucket: BUCKET_NAME,
        Key: `${id}.json`,
      })
    );

    if (!Body) return null;

    // @ts-ignore
    const buffer: string = Body.read();
    return JSON.parse(buffer);
  } catch (e) {
    return null;
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
      return projects;
    },
  })
  .query("project", {
    input: z.object({
      id: z.number(),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      return project;
    },
  })
  .query("world", {
    input: z.object({
      id: z.number(),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new trpc.TRPCError({ code: "NOT_FOUND" });

      const world = await getWorld(id);
      return world;
    },
  })
  .mutation("create-project", {
    input: z.object({
      name: z.string().max(255),
      description: z.string().max(2040),
    }),
    async resolve({ ctx: { address }, input: { name, description } }) {
      const project = await prisma.project.create({
        data: {
          owner: address,
          name,
          description,
          image: null,
          scene: null,
          studioState: null,
        },
      });

      return project;
    },
  })
  .mutation("save-project", {
    input: z.object({
      id: z.number(),
      name: z.string().max(255).optional(),
      description: z.string().max(2040).optional(),
      image: z.string().optional(),
      studioState: z.string().optional(),
      world: z.any(),
    }),
    async resolve({
      ctx: { address },
      input: { id, name, description, image, studioState, world },
    }) {
      // Verify that the user owns the project
      const project = await prisma.project.findFirst({
        where: { id, owner: address },
      });
      if (!project) throw new trpc.TRPCError({ code: "UNAUTHORIZED" });

      // Upload world to S3
      await uploadWorld(world, id);

      // Save to database
      await prisma.project.update({
        where: { id },
        data: {
          name,
          description,
          image,
          studioState,
          updatedAt: new Date(),
        },
      });

      return {
        success: true,
      };
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
