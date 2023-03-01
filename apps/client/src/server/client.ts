import { S3Client } from "@aws-sdk/client-s3";

import { env } from "../env/server.mjs";

const host = env.S3_ENDPOINT.split(":")[0];
const http = host === "localhost" || host === "127.0.0.1" ? "http" : "https";

export const s3Client = new S3Client({
  endpoint: `${http}://${env.S3_ENDPOINT}`,
  region: env.S3_REGION,
  credentials: {
    accessKeyId: env.S3_ACCESS_KEY_ID,
    secretAccessKey: env.S3_SECRET,
  },
});
