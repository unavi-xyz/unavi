import { cache } from "react";

import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { db } from "../db/drizzle";

export const fetchWorldURI = cache(async (id: string) => {
  try {
    const found = await db.query.world.findFirst({
      where: (row, { eq }) => eq(row.publicId, id),
      with: { model: { columns: { key: true } } },
    });
    if (!found) throw new Error("World not found");

    const uri = cdnURL(S3Path.worldModel(found.model.key).metadata);

    return { uri };
  } catch {
    return null;
  }
});
