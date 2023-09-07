import { GlobalDatabaseUserAttributes } from "lucia";

import { auth } from "../auth/lucia";
import { DID_ID_LENGTH } from "../db/constants";
import { db } from "../db/drizzle";
import { profile } from "../db/schema";
import { createDidWeb } from "../did";
import { nanoidLowercase } from "../nanoid";
import { genUsername } from "./genUsername";

export async function createUser({
  providerId,
  providerUserId,
  address,
}: {
  providerId: string;
  providerUserId: string;
  address: string;
}) {
  // Create user
  const attributes = await createUserAttributes({ address });

  const user = await auth.createUser({
    attributes,
    key: {
      password: null,
      providerId,
      providerUserId,
    },
  });

  // Create profile
  await db.insert(profile).values({ userId: user.userId });

  return user;
}

export async function createUserAttributes(
  extra: Partial<GlobalDatabaseUserAttributes> = {}
): Promise<GlobalDatabaseUserAttributes> {
  const username = genUsername();
  const did_id = nanoidLowercase(DID_ID_LENGTH);

  // Ensure that the username and did are unique
  try {
    const found = await db.query.user.findFirst({
      where: (row, { eq, or }) =>
        or(eq(row.did_id, did_id), eq(row.username, username)),
    });
    if (found) {
      return createUserAttributes(extra);
    }
  } catch {
    // Ignore, it's probably fine?
  }

  const did = createDidWeb(did_id);

  return {
    did,
    did_id,
    username,
    ...extra,
  };
}
