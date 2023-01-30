import { DeleteObjectsCommand, GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "../../env/server.mjs";
import { s3Client } from "./client";
import { expiresIn } from "./constants";

export const PUBLICATION_FILE = {
  IMAGE: "image",
  MODEL: "model",
  METADATA: "metadata",
} as const;

export type PublicationFile = (typeof PUBLICATION_FILE)[keyof typeof PUBLICATION_FILE];

export class Publication {
  id: string;

  constructor(id: string) {
    this.id = id;
  }

  async getUpload(type: PublicationFile) {
    const Key = this.getKey(type);
    const ContentType = this.getContentType(type);
    const command = new PutObjectCommand({
      Bucket: env.S3_BUCKET,
      Key,
      ContentType,
      ACL: "public-read",
    });
    const url = await getSignedUrl(s3Client, command, { expiresIn });
    return url;
  }

  async getDownload(type: PublicationFile) {
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
          { Key: this.getKey(PUBLICATION_FILE.IMAGE) },
          { Key: this.getKey(PUBLICATION_FILE.MODEL) },
          { Key: this.getKey(PUBLICATION_FILE.METADATA) },
        ],
      },
    });

    await s3Client.send(command);
  }

  getKey(type: PublicationFile) {
    switch (type) {
      case PUBLICATION_FILE.IMAGE: {
        return `publication/${this.id}/image.jpg`;
      }

      case PUBLICATION_FILE.MODEL: {
        return `publication/${this.id}/model.glb`;
      }

      case PUBLICATION_FILE.METADATA: {
        return `publication/${this.id}/metadata.json`;
      }
    }
  }

  getContentType(type: PublicationFile) {
    switch (type) {
      case PUBLICATION_FILE.IMAGE: {
        return "image/jpeg";
      }

      case PUBLICATION_FILE.MODEL: {
        return "model/gltf-binary";
      }

      case PUBLICATION_FILE.METADATA: {
        return "application/json";
      }
    }
  }
}
