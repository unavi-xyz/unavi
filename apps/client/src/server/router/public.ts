import { nanoid } from "nanoid";

import { createTempFileUploadURL } from "../s3";
import { createRouter } from "./context";

export const publicRouter = createRouter().mutation("get-temp-upload", {
  async resolve() {
    // Get temp file upload URL from S3
    const fileId = nanoid();
    const url = await createTempFileUploadURL(fileId);
    return { url, fileId };
  },
});
