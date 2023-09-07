import { getResolver } from "@unavi/web-did-resolver";
import { Resolver } from "did-resolver";

import { env } from "@/src/env.mjs";
import { Identity } from "@/src/utils/parseIdentity";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { db } from "../db/drizzle";
import { CustomDIDDocumentSchema } from "../did";

type BaseProfile = {
  did: string;
  bio?: string;
  address?: string;
  image?: string;
  background?: string;
};

type WithImage = {
  imageKey: string;
  image: string;
};

type WithoutImage = {
  imageKey: undefined;
  image: undefined;
};

type WithBackground = {
  backgroundKey: string;
  background: string;
};

type WithoutBackground = {
  backgroundKey: undefined;
  background: undefined;
};

type DBProfile = BaseProfile & {
  type: "db";
  id: string;
  username: string;
} & (WithImage | WithoutImage) &
  (WithBackground | WithoutBackground);

type DIDProfile = BaseProfile & {
  type: "did";
};

export type IdentityProfile = DBProfile | DIDProfile;

type ClientDBProfile = Omit<DBProfile, "id">;
export type ClientIdentityProfile = ClientDBProfile | DIDProfile;

type DBUserQuery = {
  address: string | null;
  did: string;
  id: string;
  username: string;
  profile: {
    backgroundKey: string | null;
    bio: string | null;
    imageKey: string | null;
  };
};

function dbUserToProfile(user: DBUserQuery): DBProfile {
  let bg: WithBackground | WithoutBackground = {
    background: undefined,
    backgroundKey: undefined,
  };
  let img: WithImage | WithoutImage = { image: undefined, imageKey: undefined };

  if (user.profile.backgroundKey) {
    const backgroundKey = user.profile.backgroundKey;
    const background = cdnURL(
      S3Path.profile(user.id).background(user.profile.backgroundKey)
    );
    bg = { background, backgroundKey };
  }

  if (user.profile.imageKey) {
    const imageKey = user.profile.imageKey;
    const image = cdnURL(S3Path.profile(user.id).image(user.profile.imageKey));
    img = { image, imageKey };
  }

  return {
    ...bg,
    ...img,
    address: user.address ?? undefined,
    bio: user.profile?.bio || undefined,
    did: user.did,
    id: user.id,
    type: "db",
    username: user.username,
  };
}

async function queryUsername(username: string): Promise<DBProfile | null> {
  const foundUser = await db.query.user.findFirst({
    columns: { address: true, did: true, id: true, username: true },
    where: (row, { eq }) => eq(row.username, username),
    with: {
      profile: {
        columns: { backgroundKey: true, bio: true, imageKey: true },
      },
    },
  });

  if (!foundUser) return null;

  return dbUserToProfile(foundUser);
}

async function queryAddress(address: string): Promise<DBProfile | null> {
  const foundUser = await db.query.user.findFirst({
    columns: { address: true, did: true, id: true, username: true },
    where: (row, { eq }) => eq(row.address, address),
    with: {
      profile: {
        columns: { backgroundKey: true, bio: true, imageKey: true },
      },
    },
  });

  if (!foundUser) return null;

  return dbUserToProfile(foundUser);
}

async function queryDid(did: string): Promise<DBProfile | null> {
  const foundUser = await db.query.user.findFirst({
    columns: { address: true, did: true, id: true, username: true },
    where: (row, { eq }) => eq(row.did, did),
    with: {
      profile: {
        columns: { backgroundKey: true, bio: true, imageKey: true },
      },
    },
  });

  if (!foundUser) return null;

  return dbUserToProfile(foundUser);
}

async function resolveDid(did: string): Promise<DIDProfile | null> {
  const UNSAFE_USE_HTTP =
    env.NODE_ENV === "development" && did.startsWith("did:web:localhost");
  const webResolver = getResolver({ UNSAFE_USE_HTTP });
  const didResolver = new Resolver({ ...webResolver });

  try {
    const doc = await didResolver.resolve(did);

    const parsed = CustomDIDDocumentSchema.safeParse(doc.didDocument);
    const profile = parsed.success ? parsed.data.profile : undefined;

    return {
      address: profile?.address,
      background: profile?.background,
      bio: profile?.bio,
      did,
      image: profile?.image,
      type: "did",
    };
  } catch (e) {
    console.warn("Failed to resolve DID", did, e);
    return null;
  }
}

export async function fetchProfile(
  identity: Identity
): Promise<IdentityProfile | null> {
  switch (identity.type) {
    case "username": {
      return queryUsername(identity.username);
    }
    case "address": {
      return queryAddress(identity.address);
    }
    case "did": {
      const [db, did] = await Promise.all([
        queryDid(identity.did),
        resolveDid(identity.did),
      ]);

      if (db) {
        return db;
      }

      return did;
    }
    default: {
      return null;
    }
  }
}
