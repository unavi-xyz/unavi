import { cache } from "react";

import { getProjectDownloadURL } from "@/app/api/projects/[id]/files/[file]/files";

import { getUserSession } from "../auth/getUserSession";
import { prisma } from "../prisma";

export const fetchProject = cache(async (id: string): Promise<Project | null> => {
  try {
    const imagePromise = getProjectDownloadURL(id, "image");
    const modelPromise = getProjectDownloadURL(id, "model");

    const session = await getUserSession();
    if (!session) throw new Error("Unauthorized");

    // Verify user owns the project
    const project = await prisma.project.findFirst({
      include: { Space: true },
      where: { ownerId: session.user.userId, publicId: id },
    });
    if (!project) throw new Error("Not found");

    return {
      description: project.description,
      id: project.publicId,
      image: await imagePromise,
      model: await modelPromise,
      ownerId: project.ownerId,
      spaceId: project.Space ? project.Space.publicId : null,
      title: project.title,
    };
  } catch {
    return null;
  }
});

export type Project = {
  id: string;
  title: string;
  description: string;
  spaceId: string | null;
  ownerId: string;
  image: string;
  model: string;
};
