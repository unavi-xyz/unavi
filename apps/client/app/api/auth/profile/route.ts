import { ProfileMetadata } from "@wired-protocol/types";
import { eq } from "drizzle-orm";
import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

import { auth } from "@/src/server/auth/lucia";
import { db } from "@/src/server/db/drizzle";
import { profile, user as userTable } from "@/src/server/db/schema";

import { UpdateProfileSchema } from "./types";

/**
 * Get user's profile
 */
export async function GET(request: NextRequest) {
  const authRequest = auth.handleRequest({ cookies, request });
  const session = await authRequest.validate();
  if (!session) return new Response(null, { status: 401 });

  const found = await db.query.profile.findFirst({
    where: eq(profile.userId, session.user.userId),
  });
  if (!found) return new Response(null, { status: 404 });

  const json: ProfileMetadata = {
    background: found.backgroundKey ?? undefined,
    bio: found.bio ?? undefined,
    image: found.imageKey ?? undefined,
  };

  return NextResponse.json(json);
}

/**
 * Update profile
 */
export async function PATCH(request: NextRequest) {
  const parsed = UpdateProfileSchema.safeParse(await request.json());
  if (!parsed.success)
    return new Response(JSON.stringify(parsed.error), { status: 400 });

  const authRequest = auth.handleRequest({ cookies, request });
  const session = await authRequest.validate();
  if (!session) return new Response(null, { status: 401 });

  const { username, bio, imageKey, backgroundKey } = parsed.data;

  await db.transaction(async (tx) => {
    await tx
      .insert(profile)
      .values({
        backgroundKey,
        bio,
        imageKey,
        userId: session.user.userId,
      })
      .onDuplicateKeyUpdate({
        set: {
          backgroundKey,
          bio,
          imageKey,
        },
      });

    if (username) {
      await tx
        .update(userTable)
        .set({ username })
        .where(eq(userTable.id, session.user.userId));
    }
  });

  return new Response(null, { status: 200 });
}
