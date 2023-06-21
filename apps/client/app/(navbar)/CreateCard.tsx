import { WorldMetadata } from "@wired-protocol/types";
import { redirect } from "next/navigation";

import { HOME_SERVER } from "@/src/constants";
import { env } from "@/src/env.mjs";
import { getUserSession } from "@/src/server/auth/getUserSession";
import { db } from "@/src/server/db/drizzle";
import { world, worldModel } from "@/src/server/db/schema";
import { nanoidShort } from "@/src/server/nanoid";
import { s3Client } from "@/src/server/s3";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { getWorldModelFileUploadCommand } from "../api/worlds/[id]/model/files/[file]/files";
import CreateCardButton from "./CreateCardButton";

export const runtime = "edge";

export async function createWorld() {
  "use server";

  const session = await getUserSession();
  if (!session) return;

  const publicId = nanoidShort();

  try {
    // Create world
    await db
      .insert(world)
      .values({ ownerId: session.user.userId, publicId })
      .execute();

    const found = await db.query.world.findFirst({
      columns: { id: true },
      where: (row, { eq }) => eq(row.publicId, publicId),
    });
    if (!found) return;

    // Create world model
    const modelKey = nanoidShort();

    await Promise.all([
      db
        .insert(worldModel)
        .values({ key: modelKey, worldId: found.id })
        .execute(),
      createMetadata(modelKey, `${session.user.username}@${HOME_SERVER}`),
      uploadDefaultImage(modelKey),
      uploadDefaultModel(modelKey),
    ]);
  } catch (error) {
    console.error(error);
    return;
  }

  redirect(`/world/${publicId}`);
}

async function createMetadata(publicId: string, author: string) {
  const json: WorldMetadata = {
    info: {
      authors: [author],
      host: env.NEXT_PUBLIC_DEFAULT_HOST,
      image: cdnURL(S3Path.worldModel(publicId).image),
      name: "New World",
    },
    model: cdnURL(S3Path.worldModel(publicId).model),
  };

  const blob = new Blob([JSON.stringify(json)], { type: "application/json" });
  const buffer = await blob.arrayBuffer();
  const array = new Uint8Array(buffer);

  const command = getWorldModelFileUploadCommand(publicId, "metadata", array);
  await s3Client.send(command);
}

async function uploadDefaultImage(publicId: string) {
  const res = await fetch(
    `${env.NEXT_PUBLIC_DEPLOYED_URL}/images/Default-World.jpg`
  );
  const buffer = await res.arrayBuffer();
  const array = new Uint8Array(buffer);

  const command = getWorldModelFileUploadCommand(publicId, "image", array);
  await s3Client.send(command);
}

async function uploadDefaultModel(publicId: string) {
  const res = await fetch(
    `${env.NEXT_PUBLIC_DEPLOYED_URL}/models/Default-World.glb`
  );
  const buffer = await res.arrayBuffer();
  const array = new Uint8Array(buffer);

  const command = getWorldModelFileUploadCommand(publicId, "model", array);
  await s3Client.send(command);
}

export default function CreateCard() {
  return (
    <section className="relative w-full overflow-hidden rounded-3xl bg-gradient-to-tl from-orange-300 to-yellow-200 p-8">
      <div className="absolute -bottom-4 -left-8 -rotate-12 select-none text-8xl opacity-50 md:text-9xl">
        🏗️
      </div>

      <div className="flex h-full flex-col items-center justify-center space-y-2">
        <h2 className="z-10 text-center text-3xl font-bold">
          Create your world. <br /> Share it with others.
        </h2>

        <form action={createWorld}>
          <CreateCardButton />
        </form>
      </div>
    </section>
  );
}
