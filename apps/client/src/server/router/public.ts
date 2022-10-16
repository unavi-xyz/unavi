import { nanoid } from "nanoid";

import { prisma } from "../prisma";
import { createTempFileUploadURL } from "../s3";
import { createRouter } from "./context";

export const publicRouter = createRouter()
  .mutation("get-temp-upload", {
    async resolve() {
      // Get temp file upload URL from S3
      const fileId = nanoid();
      const url = await createTempFileUploadURL(fileId);
      return { url, fileId };
    },
  })

  .query("hot-spaces", {
    resolve: async () => {
      const spaces = await prisma.space.findMany({
        orderBy: { viewsCount: "desc" },
        take: 10,
      });

      const spaceIds = spaces.map((space) => space.publicationId);
      return spaceIds;
    },
  })

  .query("hot-avatars", {
    resolve: async () => {
      const avatars = await prisma.avatar.findMany({
        orderBy: { viewsCount: "desc" },
        take: 10,
      });

      const avatarIds = avatars.map((avatar) => avatar.publicationId);
      return avatarIds;
    },
  });
