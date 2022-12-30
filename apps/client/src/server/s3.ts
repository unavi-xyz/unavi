import {
  DeleteObjectsCommand,
  GetObjectCommand,
  PutObjectCommand,
  S3Client,
} from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

import { env } from "../env/server.mjs";

const host = env.S3_ENDPOINT.split(":")[0];
const http = host === "localhost" || host === "127.0.0.1" ? "http" : "https";

const s3Client = new S3Client({
  endpoint: `${http}://${env.S3_ENDPOINT}`,
  region: env.S3_REGION,
  credentials: {
    accessKeyId: env.S3_ACCESS_KEY_ID,
    secretAccessKey: env.S3_SECRET,
  },
});

function projectImageKey(projectId: string) {
  return `projects/${projectId}/image.jpg`;
}

function projectSceneKey(projectId: string) {
  return `projects/${projectId}/scene.json`;
}

function projectFileKey(projectId: string, fileId: string) {
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

function profileMetadataKey(profileId: string) {
  return `profiles/${profileId}/metadata.json`;
}

function tempFileKey(fileId: string) {
  return `temp/${fileId}`;
}

export async function createSceneUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: projectSceneKey(projectId),
    ContentType: "application/json",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function createImageUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: projectImageKey(projectId),
    ContentType: "image/jpeg",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function createFileUploadURL(projectId: string, fileId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: projectFileKey(projectId, fileId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function createPublishedModelUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: publishedModelKey(projectId),
    ContentType: "model/gltf-binary",
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function createPublishedImageUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: publishedImageKey(projectId),
    ContentType: "image/jpeg",
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function createPublishedMetadataUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: publishedMetadataKey(projectId),
    ContentType: "application/json",
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function createProfileMetadataUploadURL(profileId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: profileMetadataKey(profileId),
    ContentType: "application/json",
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function createProfileImageUploadURL(profileId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: `profiles/${profileId}/image.jpg`,
    ContentType: "image/jpeg",
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function createProfileCoverImageUploadURL(profileId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: `profiles/${profileId}/cover.jpg`,
    ContentType: "image/jpeg",
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function createTempFileUploadURL(fileId: string) {
  const command = new PutObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: tempFileKey(fileId),
    ACL: "public-read",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function getSceneURL(projectId: string) {
  const command = new GetObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: projectSceneKey(projectId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function getImageURL(projectId: string) {
  const command = new GetObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: projectImageKey(projectId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function getFileURL(projectId: string, fileId: string) {
  const command = new GetObjectCommand({
    Bucket: env.S3_BUCKET,
    Key: projectFileKey(projectId, fileId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 600 });
  return url;
}

export async function deleteProjectFromS3(projectId: string, fileIds: string[]) {
  const fileKeys = fileIds.map((fileId) => ({
    Key: projectFileKey(projectId, fileId),
  }));

  await s3Client.send(
    new DeleteObjectsCommand({
      Bucket: env.S3_BUCKET,
      Delete: {
        Objects: [
          { Key: projectSceneKey(projectId) },
          { Key: projectImageKey(projectId) },
          ...fileKeys,
        ],
      },
    })
  );
}

export async function deleteFilesFromS3(projectId: string, fileIds: string[]) {
  const fileKeys = fileIds.map((fileId) => ({
    Key: projectFileKey(projectId, fileId),
  }));

  await s3Client.send(
    new DeleteObjectsCommand({
      Bucket: env.S3_BUCKET,
      Delete: { Objects: fileKeys },
    })
  );
}

export async function deletePublicationFromS3(publicationId: string) {
  await s3Client.send(
    new DeleteObjectsCommand({
      Bucket: env.S3_BUCKET,
      Delete: {
        Objects: [
          { Key: publishedModelKey(publicationId) },
          { Key: publishedImageKey(publicationId) },
          { Key: publishedMetadataKey(publicationId) },
        ],
      },
    })
  );
}
