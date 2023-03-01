import { getServerSession } from "../../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../../src/server/prisma";
import { Params, paramsSchema } from "../types";
import { putSchema } from "./types";

// Link publication
export async function PUT(request: Request, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);
  const { spaceId } = putSchema.parse(await request.json());

  // Verify user owns the publication
  const found = await prisma.publication.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) return new Response("Publication not found", { status: 404 });

  // Link publication to space
  await prisma.publication.update({ where: { id }, data: { spaceId } });
}
