import { DeleteObjectsCommand } from "@aws-sdk/client-s3";
import { eq } from "drizzle-orm";
import { NextRequest, NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { getUserSession } from "@/src/server/auth/getUserSession";
import { db } from "@/src/server/db/drizzle";
import { worldModel } from "@/src/server/db/schema";
import { FixWith } from "@/src/server/db/types";
import { listObjectsRecursive } from "@/src/server/helpers/listObjectsRecursive";
import { nanoidShort } from "@/src/server/nanoid";
import { s3Client } from "@/src/server/s3";
import { S3Path } from "@/src/utils/s3Paths";

import { Params, paramsSchema } from "../types";
import { PostWorldModelResponse } from "./types";

// Create new world model
export async function POST(request: NextRequest, { params }: Params) {
  const session = await getUserSession();
  if (!session) return new Response("Unauthorized", { status: 401 });

  const { id } = paramsSchema.parse(params);

  // Verify user owns the world
  const _found = await db.query.world.findFirst({
    where: (row, { eq }) =>
      eq(row.ownerId, session.user.userId) && eq(row.publicId, id),
    with: { model: true },
  });
  if (!_found) return new Response("World not found", { status: 404 });
  const found: FixWith<typeof _found, "model"> = _found;
  if (!found.model) return new Response("World not found", { status: 404 });

  // Remove existing world model
  const allObjects = await listObjectsRecursive(
    S3Path.worldModel(found.model.key).directory
  );

  await Promise.all([
    // Remove objects from S3
    allObjects.length > 0
      ? s3Client.send(
          new DeleteObjectsCommand({
            Bucket: env.S3_BUCKET,
            Delete: { Objects: allObjects.map((obj) => ({ Key: obj.Key })) },
          })
        )
      : null,

    // Remove from database
    db.delete(worldModel).where(eq(worldModel.id, found.model.id)).execute(),
  ]);

  // Create new world model
  const modelId = nanoidShort();

  await db
    .insert(worldModel)
    .values({ key: modelId, worldId: found.id })
    .execute();

  const json: PostWorldModelResponse = { modelId };
  return NextResponse.json(json);
}
