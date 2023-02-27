import { NextResponse } from "next/server";

import { getServerSession } from "../../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../../src/server/prisma";
import { getDownload } from "../../files";
import { Params } from "../types";
import { GetFileDownloadResponse, paramsSchema } from "./types";

// Get file download URL
export async function GET(request: Request, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) throw new Error("Unauthorized");

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) throw new Error("Not found");

  const url = await getDownload(id, file);

  const json: GetFileDownloadResponse = { url };
  return NextResponse.json(json);
}
