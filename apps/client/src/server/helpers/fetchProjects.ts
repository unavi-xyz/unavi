import { cache } from "react";

import { prisma } from "../prisma";
import { Project } from "../s3/Project";
import { getServerSession } from "./getServerSession";

export const fetchProjects = cache(async () => {
  const session = await getServerSession();
  if (!session) throw new Error("Not authenticated");

  const projects = await prisma.project.findMany({
    where: { owner: session.address },
    include: { Publication: true },
    orderBy: { updatedAt: "desc" },
  });

  const images = await Promise.all(
    projects.map(({ id }) => {
      const project = new Project(id);
      return project.getDownload("image");
    })
  );

  const response = projects.map((project, index) => ({
    ...project,
    image: images[index],
  }));

  return response;
});
