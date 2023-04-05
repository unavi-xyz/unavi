import { DeleteObjectsCommand } from "@aws-sdk/client-s3";
import { NextRequest } from "next/server";

import { env } from "@/src/env.mjs";
import { getServerSession } from "@/src/server/helpers/getServerSession";
import { listObjectsRecursive } from "@/src/server/helpers/listObjectsRecursive";
import { prisma } from "@/src/server/prisma";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

import { Params, paramsSchema } from "./types";

// Delete publication
export async function DELETE(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the publication
  const found = await prisma.publication.findFirst({ where: { id, owner: session.address } });
  if (!found) return new Response("Publication not found", { status: 404 });

  const allObjects = await listObjectsRecursive(S3Path.publication(id).directory);

  await Promise.all([
    // Delete files from S3
    s3Client.send(
      new DeleteObjectsCommand({
        Bucket: env.S3_BUCKET,
        Delete: { Objects: allObjects.map(({ Key }) => ({ Key })) },
      })
    ),

    // Delete publication from database
    prisma.publication.delete({ where: { id } }),
  ]);

  return new Response("OK", { status: 200 });
}
