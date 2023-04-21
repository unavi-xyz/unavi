import { cache } from "react";

import { getProjectDownloadURL } from "@/app/api/projects/[id]/files/[file]/files";

import { prisma } from "../prisma";
import { getServerSession } from "./getServerSession";

export const fetchProjects = cache(async (): Promise<Project[]> => {
  const session = await getServerSession();
  if (!session) throw new Error("Unauthorized");

  const projects = await prisma.project.findMany({
    where: { owner: session.address },
    orderBy: { updatedAt: "desc" },
  });

  const images = await Promise.all(
    projects.map(({ publicId }) => getProjectDownloadURL(publicId, "image"))
  );

  const response: Project[] = projects.map((project, index) => ({
    createdAt: project.createdAt,
    description: project.description,
    image: images[index],
    title: project.title,
    owner: project.owner,
    publicId: project.publicId,
    spaceId: project.spaceId,
    updatedAt: project.updatedAt,
  }));

  return response;
});

export type Project = {
  createdAt: Date;
  description: string;
  image?: string;
  title: string;
  owner: string;
  publicId: string;
  spaceId: number | null;
  updatedAt: Date;
};
