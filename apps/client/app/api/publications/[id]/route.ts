import { DeleteObjectCommand } from "@aws-sdk/client-s3";
import { NextRequest } from "next/server";

import { pathAsset } from "../../../../src/editor/utils/s3Paths";
import { env } from "../../../../src/env/server.mjs";
import { s3Client } from "../../../../src/server/client";
import { getServerSession } from "../../../../src/server/helpers/getServerSession";
import { prisma } from "../../../../src/server/prisma";
import { deleteFiles } from "../files";
import { Params, paramsSchema } from "./types";

// Delete publication
export async function DELETE(request: NextRequest, { params }: Params) {
  const session = await getServerSession();
  if (!session || !session.address) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the publication
  const found = await prisma.publication.findFirst({
    where: { id, owner: session.address },
    include: { PublicationAsset: { where: { Asset: { projectId: null } } } },
  });
  if (!found) return new Response("Publication not found", { status: 404 });

  await Promise.all([
    // Delete asset files from S3
    Promise.all(
      found.PublicationAsset.map((publicationAsset) => {
        const command = new DeleteObjectCommand({
          Bucket: env.S3_BUCKET,
          Key: pathAsset(publicationAsset.assetId),
        });
        return s3Client.send(command);
      })
    ),

    // Delete publication from database
    prisma.publication.delete({
      where: { id },
      include: {
        ViewEvents: true,
        PublicationAsset: {
          where: {
            Asset: {
              projectId: null,
            },
          },
        },
      },
    }),

    // Delete files from S3
    deleteFiles(id),
  ]);
}
