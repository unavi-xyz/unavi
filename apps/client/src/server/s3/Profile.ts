import { GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "../../env/server.mjs";
import { s3Client } from "./client";
import { expiresIn } from "./constants";

export const PROFILE_FILE = {
  IMAGE: "image",
  COVER: "cover",
  METADATA: "metadata",
} as const;

export type ProfileFile = (typeof PROFILE_FILE)[keyof typeof PROFILE_FILE];

export class Profile {
  id: string;

  constructor(id: string) {
    this.id = id;
  }

  async getUpload(type: ProfileFile) {
    const Key = this.getKey(type);
    const ContentType = this.getContentType(type);
    const command = new PutObjectCommand({ Bucket: env.S3_BUCKET, Key, ContentType });
    const url = await getSignedUrl(s3Client, command, { expiresIn });
    return url;
  }

  async getDownload(type: ProfileFile) {
    const Key = this.getKey(type);
    const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });
    const url = await getSignedUrl(s3Client, command, { expiresIn });
    return url;
  }

  getKey(type: ProfileFile) {
    switch (type) {
      case PROFILE_FILE.IMAGE: {
        return `profiles/${this.id}/image.jpg`;
      }

      case PROFILE_FILE.COVER: {
        return `profiles/${this.id}/cover.jpg`;
      }

      case PROFILE_FILE.METADATA: {
        return `profiles/${this.id}/metadata.json`;
      }
    }
  }

  getContentType(type: ProfileFile) {
    switch (type) {
      case PROFILE_FILE.IMAGE: {
        return "image/jpeg";
      }

      case PROFILE_FILE.COVER: {
        return "image/jpeg";
      }

      case PROFILE_FILE.METADATA: {
        return "application/json";
      }
    }
  }
}
