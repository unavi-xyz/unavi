import { _Object, ListObjectsV2Command } from "@aws-sdk/client-s3";

import { env } from "../../env.mjs";
import { s3Client } from "../s3";

/**
 * Get all keys recurively
 */
export async function listObjectsRecursive(
  Prefix: string,
  ContinuationToken?: string
): Promise<_Object[]> {
  // Get objects for current prefix
  const listObjects = await s3Client.send(
    new ListObjectsV2Command({
      Bucket: env.S3_BUCKET,
      Prefix,
      ContinuationToken,
    })
  );

  let deepFiles, nextFiles;

  // Recurive call to get sub prefixes
  if (listObjects.CommonPrefixes) {
    const deepFilesPromises = listObjects.CommonPrefixes.flatMap(({ Prefix }) => {
      if (!Prefix) return [];
      return listObjectsRecursive(Prefix);
    });

    deepFiles = (await Promise.all(deepFilesPromises)).flatMap((t) => t);
  }

  // If we must paginate
  if (listObjects.IsTruncated) {
    nextFiles = await listObjectsRecursive(Prefix, listObjects.NextContinuationToken);
  }

  return [...(listObjects.Contents || []), ...(deepFiles || []), ...(nextFiles || [])];
}
