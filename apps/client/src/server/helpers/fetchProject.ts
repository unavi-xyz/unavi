import { cache } from "react";

import { getProjectDownloadURL } from "@/app/api/projects/[id]/files/[file]/files";

import { prisma } from "../prisma";
import { getServerSession } from "./getServerSession";

export const fetchProject = cache(async (id: string): Promise<Project | null> => {
  try {
    const imagePromise = getProjectDownloadURL(id, "image");
    const modelPromise = getProjectDownloadURL(id, "model");

    const session = await getServerSession();
    if (!session) throw new Error("Unauthorized");

    // Verify user owns the project
    const project = await prisma.project.findFirst({
      where: { publicId: id, owner: session.address },
      include: { Space: true },
    });
    if (!project) throw new Error("Not found");

    return {
      id: project.publicId,
      title: project.title,
      description: project.description,
      spaceId: project.Space ? project.Space.publicId : null,
      owner: project.owner,
      image: await imagePromise,
      model: await modelPromise,
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
  owner: string;
  image: string;
  model: string;
};
