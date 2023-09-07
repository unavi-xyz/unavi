import { User } from "lucia";
import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

import { validateEthereumAuth } from "@/src/server/auth/ethereum";
import { auth } from "@/src/server/auth/lucia";
import { AuthMethod, AuthSchema } from "@/src/server/auth/types";
import { createUser } from "@/src/server/helpers/createUser";

import { LoginResponse } from "./types";

/**
 * User login
 */
export async function POST(request: NextRequest) {
  const parsedInput = AuthSchema.safeParse(await request.json());
  if (!parsedInput.success) {
    return NextResponse.json({ error: "Invalid input" }, { status: 400 });
  }

  // Validate signature
  const result = await validateEthereumAuth(request, parsedInput.data);
  if (!result) {
    return NextResponse.json({ error: "Invalid signature" }, { status: 400 });
  }

  let user: User;

  try {
    // Try to get existing user
    const key = await auth.useKey(
      AuthMethod.Ethereum,
      result.data.address,
      null
    );
    user = await auth.getUser(key.userId);
  } catch {
    // Create new user if it doesn't exist
    user = await createUser({
      address: result.data.address,
      providerId: parsedInput.data.method,
      providerUserId: result.data.address,
    });
  }

  // Create auth session
  const authRequest = auth.handleRequest({ cookies, request });
  const session = await auth.createSession({
    attributes: {},
    userId: user.userId,
  });
  authRequest.setSession(session);

  const json: LoginResponse = { user };
  return NextResponse.json(json);
}
