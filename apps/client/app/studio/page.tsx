import { redirect } from "next/navigation";

import { getSession } from "@/src/server/auth/getSession";
import { nanoidShort } from "@/src/server/nanoid";
import { prisma } from "@/src/server/prisma";

export default async function Page() {
  const session = await getSession();
  if (!session) redirect("/");

  // Get the user's most recent project
  let dbProject = await prisma.project.findFirst({
    where: { ownerId: session.userId },
    orderBy: { updatedAt: "desc" },
    select: { publicId: true },
  });

  // If they don't have any projects, create a new one
  if (!dbProject) {
    dbProject = await prisma.project.create({
      data: {
        ownerId: session.userId,
        publicId: nanoidShort(),
        title: "New Space",
        description: "A space for you to create and share.",
      },
      select: { publicId: true },
    });
  }

  // Redirect to the project
  redirect(`/studio/${dbProject.publicId}`);
}
