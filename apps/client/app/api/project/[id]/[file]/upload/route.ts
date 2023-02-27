import { NextResponse } from "next/server";

import { getServerSession } from "../../../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../../../src/server/prisma";
import { getUpload } from "../../../files";
import { Params, paramsSchema } from "../types";
import { GetFileUploadResponse } from "./types";

// Get file upload URL
export async function GET(request: Request, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) throw new Error("Unauthorized");

  const { id, file } = paramsSchema.parse(params);

  // Verify user owns the project
  const found = await prisma.project.findFirst({
    where: { id, owner: session.address },
  });
  if (!found) throw new Error("Not found");

  const url = await getUpload(id, file);

  const json: GetFileUploadResponse = { url };
  return NextResponse.json(json);
}
