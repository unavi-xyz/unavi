import { cache } from "react";

import { getProjectDownloadURL } from "@/app/api/projects/[id]/files/[file]/files";

import { prisma } from "../prisma";
import { getServerSession } from "./getServerSession";

export const fetchProjects = cache(async () => {
  const session = await getServerSession();
  if (!session) throw new Error("Unauthorized");

  const projects = await prisma.project.findMany({
    where: { owner: session.address },
    orderBy: { updatedAt: "desc" },
  });

  const images = await Promise.all(
    projects.map(({ publicId }) => getProjectDownloadURL(publicId, "image"))
  );

  const response = projects.map((project, index) => ({
    ...project,
    image: images[index],
  }));

  return response;
});
