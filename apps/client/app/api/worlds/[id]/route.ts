import { DeleteObjectsCommand } from "@aws-sdk/client-s3";
import { eq } from "drizzle-orm";
import { NextRequest, NextResponse } from "next/server";

import { env } from "@/src/env.mjs";
import { getUserSession } from "@/src/server/auth/getUserSession";
import { db } from "@/src/server/db/drizzle";
import { world, worldModel } from "@/src/server/db/schema";
import { FixWith } from "@/src/server/db/types";
import { listObjectsRecursive } from "@/src/server/helpers/listObjectsRecursive";
import { s3Client } from "@/src/server/s3";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { GetResponse, Params, paramsSchema } from "./types";

// Get world
export async function GET(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);

  const _found = await db.query.world.findFirst({
    where: (row, { eq }) => eq(row.publicId, id),
    with: { model: true },
  });
  if (!_found) return new Response("World not found", { status: 404 });
  const found: FixWith<typeof _found, "model"> = _found;
  if (!found.model) return new Response("World not found", { status: 404 });

  const modelURI = cdnURL(S3Path.worldModel(found.model.key).model);

  const json: GetResponse = { ownerId: found.ownerId, uri: modelURI };
  return NextResponse.json(json);
}

// Delete world
export async function DELETE(request: NextRequest, { params }: Params) {
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

  // Delete files from S3
  const objectsPromise = listObjectsRecursive(
    S3Path.worldModel(found.model.key).directory
  ).then((objs) =>
    objs.length > 0
      ? s3Client.send(
          new DeleteObjectsCommand({
            Bucket: env.S3_BUCKET,
            Delete: { Objects: objs.map(({ Key }) => ({ Key })) },
          })
        )
      : null
  );

  await db
    .delete(worldModel)
    .where(eq(worldModel.key, found.model.key))
    .execute();

  await db.delete(world).where(eq(world.publicId, id)).execute();

  await objectsPromise;

  return NextResponse.json({ success: true });
}
