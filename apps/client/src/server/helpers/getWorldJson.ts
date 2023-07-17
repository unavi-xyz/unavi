import { WorldMetadata } from "@wired-protocol/types";

import { HOME_SERVER } from "@/src/constants";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { db } from "../db/drizzle";
import { FixWith } from "../db/types";

export async function getWorldJson(publicId: string) {
  const _found = await db.query.world.findFirst({
    columns: {
      description: true,
      host: true,
      ownerId: true,
      title: true,
    },
    where: (row, { eq }) => eq(row.publicId, publicId),
    with: { model: true },
  });
  if (!_found) return null;
  const found: FixWith<typeof _found, "model"> = _found;
  if (!found.model) return null;

  const foundUser = await db.query.user.findFirst({
    columns: { username: true },
    where: (row, { eq }) => eq(row.id, found.ownerId),
  });
  if (!foundUser) return null;

  const handle = `@${foundUser.username}:${HOME_SERVER}`;

  const image = cdnURL(S3Path.worldModel(found.model.key).image);
  const model = cdnURL(S3Path.worldModel(found.model.key).model);

  const json: WorldMetadata = {
    info: {
      authors: [handle],
      description: found.description || undefined,
      host: found.host || undefined,
      image,
      title: found.title || undefined,
    },
    model,
  };

  return json;
}
