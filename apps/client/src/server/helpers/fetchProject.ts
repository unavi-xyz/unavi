import { cache } from "react";

import { getDownload } from "../../../app/api/projects/files";
import { prisma } from "../prisma";
import { getServerSession } from "./getServerSession";

export const fetchProject = cache(async (id: string) => {
  try {
    const session = await getServerSession();
    if (!session) throw new Error("Unauthorized");

    const imagePromise = getDownload(id, "image");

    const project = await prisma.project.findFirst({
      // Verify user owns the project
      where: { id, owner: session.address },
      include: { Publication: true },
    });
    if (!project) throw new Error("Not found");

    return { ...project, image: await imagePromise };
  } catch (err) {
    console.warn("Error fetching project", id, err);
    return null;
  }
});

export type Project = Exclude<Awaited<ReturnType<typeof fetchProject>>, null>;
