import { World } from "@wired-protocol/types";

import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { db } from "../db/drizzle";

export async function getWorldJson(publicId: string) {
  const found = await db.query.world.findFirst({
    columns: {
      description: true,
      host: true,
      ownerId: true,
      title: true,
    },
    where: (row, { eq }) => eq(row.publicId, publicId),
    with: { model: true },
  });
  if (!found?.model) return null;

  const foundUser = await db.query.user.findFirst({
    columns: { did: true },
    where: (row, { eq }) => eq(row.id, found.ownerId),
  });
  if (!foundUser) return null;

  const image = cdnURL(S3Path.worldModel(found.model.key).image);
  const model = cdnURL(S3Path.worldModel(found.model.key).model);

  const json: World = {
    authors: [foundUser.did],
    description: found.description || undefined,
    host: found.host || undefined,
    image,
    model,
    title: found.title || undefined,
  };

  return json;
}
