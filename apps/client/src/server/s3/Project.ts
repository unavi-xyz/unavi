import { DeleteObjectsCommand, GetObjectCommand, PutObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "../../env/server.mjs";
import { s3Client } from "./client";
import { expiresIn } from "./constants";

export const PROJECT_FILE = {
  IMAGE: "image",
  MODEL: "model",
} as const;

export type ProjectFile = (typeof PROJECT_FILE)[keyof typeof PROJECT_FILE];

export class Project {
  id: string;

  constructor(id: string) {
    this.id = id;
  }

  async getUpload(type: ProjectFile) {
    const Key = this.getKey(type);
    const ContentType = this.getContentType(type);
    const command = new PutObjectCommand({ Bucket: env.S3_BUCKET, Key, ContentType });
    const url = await getSignedUrl(s3Client, command, { expiresIn });
    return url;
  }

  async getDownload(type: ProjectFile) {
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
          { Key: this.getKey(PROJECT_FILE.IMAGE) },
          { Key: this.getKey(PROJECT_FILE.MODEL) },
        ],
      },
    });

    await s3Client.send(command);
  }

  getKey(type: ProjectFile) {
    switch (type) {
      case PROJECT_FILE.IMAGE: {
        return `projects/${this.id}/image.jpg`;
      }

      case PROJECT_FILE.MODEL: {
        return `projects/${this.id}/model.glb`;
      }
    }
  }

  getContentType(type: ProjectFile) {
    switch (type) {
      case PROJECT_FILE.IMAGE: {
        return "image/jpeg";
      }

      case PROJECT_FILE.MODEL: {
        return "model/gltf-binary";
      }
    }
  }
}
