import { cache } from "react";

import { getDownload } from "../../../app/api/project/files";
import { prisma } from "../prisma";
import { getServerSession } from "./getServerSession";

export const fetchProjects = cache(async () => {
  const session = await getServerSession();
  if (!session) throw new Error("Unauthorized");

  const projects = await prisma.project.findMany({
    where: { owner: session.address },
    include: { Publication: true },
    orderBy: { updatedAt: "desc" },
  });

  const images = await Promise.all(projects.map(({ id }) => getDownload(id, "image")));

  const response = projects.map((project, index) => ({
    ...project,
    image: images[index],
  }));

  return response;
});
