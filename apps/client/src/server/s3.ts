import {
  DeleteObjectsCommand,
  GetObjectCommand,
  PutObjectCommand,
  S3Client,
} from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

const s3Client = new S3Client({
  endpoint: `https://${process.env.S3_ENDPOINT}` ?? "",
  region: process.env.S3_REGION ?? "",
  credentials: {
    accessKeyId: process.env.S3_ACCESS_KEY_ID ?? "",
    secretAccessKey: process.env.S3_SECRET ?? "",
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

export async function createSceneUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: process.env.S3_BUCKET,
    Key: sceneKey(projectId),
    ContentType: "application/json",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function createImageUploadURL(projectId: string) {
  const command = new PutObjectCommand({
    Bucket: process.env.S3_BUCKET,
    Key: imageKey(projectId),
    ContentType: "image/jpeg",
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function createFileUploadURL(projectId: string, fileId: string) {
  const command = new PutObjectCommand({
    Bucket: process.env.S3_BUCKET,
    Key: fileKey(projectId, fileId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function getSceneURL(projectId: string) {
  const command = new GetObjectCommand({
    Bucket: process.env.S3_BUCKET,
    Key: sceneKey(projectId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function getImageURL(projectId: string) {
  const command = new GetObjectCommand({
    Bucket: process.env.S3_BUCKET,
    Key: imageKey(projectId),
  });

  const url = await getSignedUrl(s3Client, command, { expiresIn: 300 });
  return url;
}

export async function getFileURL(projectId: string, fileId: string) {
  const command = new GetObjectCommand({
    Bucket: process.env.S3_BUCKET,
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
      Bucket: process.env.S3_BUCKET,
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
      Bucket: process.env.S3_BUCKET,
      Delete: { Objects: fileKeys },
    })
  );
}
