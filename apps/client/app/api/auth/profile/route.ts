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
  const { session } = await authRequest.validateUser();
  if (!session) return new Response(null, { status: 401 });

  const found = await db.query.profile.findFirst({
    where: eq(profile.userId, session.userId),
  });
  if (!found) return new Response(null, { status: 404 });

  const json: ProfileMetadata = {
    background: found.backgroundKey ?? undefined,
    bio: found.bio ?? undefined,
    image: found.imageKey ?? undefined,
    name: found.name ?? undefined,
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
  const { session, user } = await authRequest.validateUser();
  if (!session) return new Response(null, { status: 401 });

  const { username, name, bio, imageKey, backgroundKey } = parsed.data;

  // Create or update profile
  await db
    .insert(profile)
    .values({
      backgroundKey,
      bio,
      imageKey,
      name,
      userId: session.userId,
      username: user.username,
    })
    .onDuplicateKeyUpdate({
      set: {
        backgroundKey,
        bio,
        imageKey,
        name,
      },
    });

  // Update username
  if (username && username !== user.username) {
    // Check if username is taken
    const existingUser = await db.query.user.findFirst({
      where: (row, { eq }) => eq(row.username, username),
    });
    if (existingUser) return new Response(null, { status: 409 });

    // Update username
    await db
      .update(userTable)
      .set({ username })
      .where(eq(userTable.id, session.userId));
  }

  return new Response(null, { status: 200 });
}
