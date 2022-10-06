import {
  DeleteObjectsCommand,
  GetObjectCommand,
  PutObjectCommand,
  S3Client,
} from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "../env/server.mjs";

const s3Client = new S3Client({
  endpoint: `https://${env.S3_ENDPOINT}` ?? "",
  region: env.S3_REGION ?? "",
  credentials: {
    accessKeyId: env.S3_ACCESS_KEY_ID ?? "",
    secretAccessKey: env.S3_SECRET ?? "",
  },
});

function imageKey(projectId: string) {
  return `projects/${projectId}/image.jpg`;
}

function sceneKey(projectId: string) {
  return `projects/${projectId}/scene.json`;
}

function fileKey(projectId: string, fileId: string) {
  return `projects/${projectId}/files/${fileId}`;
}

function publishedModelKey(projectId: string) {
  return `published/${projectId}/model.glb`;
}

function publishedImageKey(projectId: string) {
  return `published/${projectId}/image.jpg`;
}

function publishedMetadataKey(projectId: string) {
  return `published/${projectId}/metadata.json`;
}

export async function createSceneUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: sceneKey(projectId),
    ContentType: "application/json",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function createImageUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: imageKey(projectId),
    ContentType: "image/jpeg",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function createFileUploadURL(projectId: string, fileId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: fileKey(projectId, fileId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function createPublishedModelUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: publishedModelKey(projectId),
    ContentType: "model/gltf-binary",
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function createPublishedImageUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: publishedImageKey(projectId),
    ContentType: "image/jpeg",
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function createPublishedMetadataUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: publishedMetadataKey(projectId),
    ContentType: "application/json",
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function getSceneURL(projectId: string) {
  const command = new GetObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: sceneKey(projectId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function getImageURL(projectId: string) {
  const command = new GetObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: imageKey(projectId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function getFileURL(projectId: string, fileId: string) {
  const command = new GetObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: fileKey(projectId, fileId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function deleteProjectFromS3(
  projectId: string,
  fileIds: string[]
) {
  const fileKeys = fileIds.map((fileId) => ({
    Key: fileKey(projectId, fileId),
  }));

  await s3Client.send(
    new DeleteObjectsCommand({
      Bucket: env.S3_BUCKET,
      Delete: {
        Objects: [
          { Key: sceneKey(projectId) },
          { Key: imageKey(projectId) },
          ...fileKeys,
        ],
      },
    })
  );
}

export async function deleteFilesFromS3(projectId: string, fileIds: string[]) {
  const fileKeys = fileIds.map((fileId) => ({
    Key: fileKey(projectId, fileId),
  }));

  await s3Client.send(
    new DeleteObjectsCommand({
      Bucket: env.S3_BUCKET,
      Delete: { Objects: fileKeys },
    })
  );
}
