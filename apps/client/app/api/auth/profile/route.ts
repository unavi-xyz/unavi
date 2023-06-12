import { ProfileMetadata } from "@wired-protocol/types";
import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

import { auth } from "@/src/server/auth/lucia";
import { prisma } from "@/src/server/prisma";

import { UpdateProfileSchema } from "./types";

/**
 * Get user's profile
 */
export async function GET(request: NextRequest) {
  const authRequest = auth.handleRequest({ cookies, request });
  const { session } = await authRequest.validateUser();
  if (!session) return new Response(null, { status: 401 });

  const profile = await prisma.profile.findUnique({
    where: { userId: session.userId },
  });
  if (!profile) return new Response(null, { status: 404 });

  const json: ProfileMetadata = {
    background: profile.background ?? undefined,
    bio: profile.bio ?? undefined,
    image: profile.image ?? undefined,
    name: profile.name ?? undefined,
  };

  return NextResponse.json(json);
}

/**
 * Update profile
 */
export async function PATCH(request: NextRequest) {
  const parsed = UpdateProfileSchema.safeParse(await request.json());
  if (!parsed.success) return new Response(JSON.stringify(parsed.error), { status: 400 });

  const authRequest = auth.handleRequest({ cookies, request });
  const { session, user } = await authRequest.validateUser();
  if (!session) return new Response(null, { status: 401 });

  const { username, name, bio, image, background } = parsed.data;

  // Create or update profile
  await prisma.profile.upsert({
    create: { background, bio, image, name, userId: session.userId },
    update: { background, bio, image, name },
    where: { userId: session.userId },
  });

  // Update username
  if (username && username !== user.username) {
    // Check if username is taken
    const existingUser = await prisma.authUser.findUnique({ where: { username } });
    if (existingUser) return new Response(null, { status: 409 });

    await prisma.authUser.update({
      data: { username },
      where: { id: session.userId },
    });
  }

  return new Response(null, { status: 200 });
}
