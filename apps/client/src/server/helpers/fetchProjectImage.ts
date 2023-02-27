import { getDownload } from "../../../app/api/project/files";
import { prisma } from "../prisma";
import { getServerSession } from "./getServerSession";

export async function fetchProjectImage(id: string) {
  const session = await getServerSession();
  if (!session) throw new Error("Unauthorized");

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) throw new Error("Not found");

  const url = await getDownload(id, "image");

  return url;
}
