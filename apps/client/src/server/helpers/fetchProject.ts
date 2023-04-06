import { cache } from "react";

import { getProjectDownload } from "@/app/api/projects/files";

import { prisma } from "../prisma";
import { getServerSession } from "./getServerSession";

export const fetchProject = cache(async (id: string) => {
  try {
    const imagePromise = getProjectDownload(id, "image");
    const modelPromise = getProjectDownload(id, "model");

    const session = await getServerSession();
    if (!session) throw new Error("Unauthorized");

    const project = await prisma.project.findFirst({
      // Verify user owns the project
      where: { id, owner: session.address },
      include: { Publication: true },
    });
    if (!project) throw new Error("Not found");

    return {
      id: project.id,
      title: project.title || project.name,
      description: project.description,
      owner: project.owner,
      publicationId: project.publicationId,
      publication: project.Publication
        ? {
            spaceId: project.Publication.spaceId,
          }
        : null,
      image: await imagePromise,
      model: await modelPromise,
    };
  } catch {
    return null;
  }
});

export type Project = Exclude<Awaited<ReturnType<typeof fetchProject>>, null>;
