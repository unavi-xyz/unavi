import { redirect } from "next/navigation";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { db } from "@/src/server/db/drizzle";
import * as schema from "@/src/server/db/schema";
import { nanoidShort } from "@/src/server/nanoid";

import CreateCardButton from "./CreateCardButton";

export async function createWorld() {
  "use server";

  const session = await getUserSession();
  if (!session) return;

  const publicId = nanoidShort();

  try {
    await db
      .insert(schema.world)
      .values({ ownerId: session.user.userId, publicId })
      .execute();
  } catch (error) {
    console.error(error);
    return;
  }

  redirect(`/space/${publicId}`);
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
