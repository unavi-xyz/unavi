import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

import { auth } from "@/src/server/auth/lucia";

import { GetAuthStatusResponse } from "./types";

export async function GET(request: NextRequest) {
  const authRequest = auth.handleRequest({ cookies, request });

  try {
    // Check if user has a valid session
    const { session, user } = await authRequest.validateUser();
    if (!session || !user) throw new Error("No valid session found");

    // User has a valid session
    const json: GetAuthStatusResponse = { status: "authenticated", user };
    return NextResponse.json(json);
  } catch {
    // User does not have a valid session
    const json: GetAuthStatusResponse = { status: "unauthenticated" };
    return NextResponse.json(json);
  }
}
