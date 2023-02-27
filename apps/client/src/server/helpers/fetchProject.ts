import { cache } from "react";

import { prisma } from "../prisma";
import { getServerSession } from "./getServerSession";

export const fetchProject = cache(async (id: string) => {
  const session = await getServerSession();
  if (!session) throw new Error("Not authenticated");

  const project = await prisma.project.findFirst({
    // Verify user owns the project
    where: { id, owner: session.address },
    include: { Publication: true },
  });

  if (!project) throw new Error("Not found");

  return project;
});
