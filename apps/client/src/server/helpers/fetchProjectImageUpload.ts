import { prisma } from "../prisma";
import { Project } from "../s3/Project";
import { getServerSession } from "./getServerSession";

export async function fetchProjectImageUpload(id: string) {
  const session = await getServerSession();
  if (!session) throw new Error("Not authenticated");

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) throw new Error("Not found");

  const project = new Project(id);
  const url = await project.getUpload("image");

  return url;
}
