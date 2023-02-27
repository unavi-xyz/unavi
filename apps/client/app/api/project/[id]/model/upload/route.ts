import { NextResponse } from "next/server";

import { getServerSession } from "../../../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../../../src/server/prisma";
import { getUpload } from "../../../files";
import { GetModelUploadResponse } from "./types";

// Get model upload URL
export async function GET(request: Request, { id }: { id: string }) {
  const session = await getServerSession();
  if (!session || !session.address) throw new Error("Not authenticated");

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) throw new Error("Not found");

  const url = await getUpload(id, "model");

  const json: GetModelUploadResponse = { url };
  return NextResponse.json(json);
}
