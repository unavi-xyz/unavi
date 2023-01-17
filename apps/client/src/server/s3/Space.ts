import { DeleteObjectsCommand, GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "../../env/server.mjs";
import { s3Client } from "./client";
import { expiresIn } from "./constants";

export const SPACE_FILE = {
  IMAGE: "image",
  MODEL: "model",
  METADATA: "metadata",
} as const;

export type SpaceFile = (typeof SPACE_FILE)[keyof typeof SPACE_FILE];

export class Space {
  id: string;

  constructor(id: string) {
    this.id = id;
  }

  async getUpload(type: SpaceFile) {
    const Key = this.getKey(type);
    const ContentType = this.getContentType(type);
    const command = new PutObjectCommand({ Bucket: env.S3_BUCKET, Key, ContentType });
    const url = await getSignedUrl(s3Client, command, { expiresIn });
    return url;
  }

  async getDownload(type: SpaceFile) {
    const Key = this.getKey(type);
    const command = new GetObjectCommand({ Bucket: env.S3_BUCKET, Key });
    const url = await getSignedUrl(s3Client, command, { expiresIn });
    return url;
  }

  async delete() {
    const command = new DeleteObjectsCommand({
      Bucket: env.S3_BUCKET,
      Delete: {
        Objects: [
          { Key: this.getKey(SPACE_FILE.IMAGE) },
          { Key: this.getKey(SPACE_FILE.MODEL) },
          { Key: this.getKey(SPACE_FILE.METADATA) },
        ],
      },
    });

    await s3Client.send(command);
  }

  getKey(type: SpaceFile) {
    switch (type) {
      case SPACE_FILE.IMAGE: {
        return `spaces/${this.id}/image.jpg`;
      }

      case SPACE_FILE.MODEL: {
        return `spaces/${this.id}/model.glb`;
      }

      case SPACE_FILE.METADATA: {
        return `spaces/${this.id}/metadata.json`;
      }
    }
  }

  getContentType(type: SpaceFile) {
    switch (type) {
      case SPACE_FILE.IMAGE: {
        return "image/jpeg";
      }

      case SPACE_FILE.MODEL: {
        return "model/gltf-binary";
      }

      case SPACE_FILE.METADATA: {
        return "application/json";
      }
    }
  }
}
