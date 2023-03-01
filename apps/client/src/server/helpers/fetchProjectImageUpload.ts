import { getUpload } from "../../../app/api/projects/files";
import { prisma } from "../prisma";
import { getServerSession } from "./getServerSession";

export async function fetchProjectImageUpload(id: string) {
  const session = await getServerSession();
  if (!session) throw new Error("Unauthorized");

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) throw new Error("Not found");

  const url = await getUpload(id, "image");

  return url;
}
