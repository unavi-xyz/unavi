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

export async function uploadSceneToS3(scene: any, id: string) {
  const data = await s3Client.send(
    new PutObjectCommand({
      Bucket: process.env.S3_BUCKET,
      Key: `${id}.json`,
      Body: JSON.stringify(scene),
      ACL: "private",
      ContentType: "application/json",
    })
  );
  return data;
}

export async function uploadImageToS3(image: any, id: string) {
  const data = await s3Client.send(
    new PutObjectCommand({
      Bucket: process.env.S3_BUCKET,
      Key: `${id}.jpeg`,
      Body: image,
      ACL: "private",
      ContentType: "image/jpeg",
    })
  );
  return data;
}
export async function uploadFileBlobToS3(
  blob: any,
  fileId: string,
  projectId: string
) {
  const data = await s3Client.send(
    new PutObjectCommand({
      Bucket: process.env.S3_BUCKET,
      Key: `${projectId}-${fileId}.blob`,
      Body: blob,
      ACL: "private",
      ContentType: "application/octet-stream",
    })
  );
  return data;
}

export async function getSceneFromS3(id: string) {
  const command = new GetObjectCommand({
    Bucket: process.env.S3_BUCKET,
    Key: `${id}.json`,
  });
  const url = await getSignedUrl(s3Client, command, { expiresIn: 3600 });
  return url;
}

export async function getImageFromS3(id: string) {
  const command = new GetObjectCommand({
    Bucket: process.env.S3_BUCKET,
    Key: `${id}.jpeg`,
  });
  const url = await getSignedUrl(s3Client, command, { expiresIn: 3600 });
  return url;
}

export async function getFileBlobFromS3(fileId: string, projectId: string) {
  const command = new GetObjectCommand({
    Bucket: process.env.S3_BUCKET,
    Key: `${projectId}-${fileId}.blob`,
  });
  const url = await getSignedUrl(s3Client, command, { expiresIn: 3600 });
  return url;
}

export async function deleteProjectFromS3(id: string) {
  await s3Client.send(
    new DeleteObjectsCommand({
      Bucket: process.env.S3_BUCKET,
      Delete: {
        Objects: [{ Key: `${id}.json` }, { Key: `${id}.jpeg` }],
      },
    })
  );
}
