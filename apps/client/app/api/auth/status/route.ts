import { NextRequest } from "next/server";

import { authJsonResponse } from "@/src/server/auth/authJsonResponse";
import { auth } from "@/src/server/auth/lucia";

import { GetAuthStatusResponse } from "./types";

export async function GET(request: NextRequest) {
  const authRequest = auth.handleRequest(request, undefined);

  try {
    // Check if user has a valid session
    const { session, user } = await authRequest.validateUser();
    if (!session || !user) throw new Error("No valid session found");

    // User has a valid session
    const json: GetAuthStatusResponse = { status: "authenticated", user };
    return authJsonResponse(json, authRequest);
  } catch {
    // User does not have a valid session
    const json: GetAuthStatusResponse = { status: "unauthenticated" };
    return authJsonResponse(json, authRequest);
  }
}
