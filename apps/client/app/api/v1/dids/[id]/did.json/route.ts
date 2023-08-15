import { DIDDocument } from "did-resolver";
import { NextRequest, NextResponse } from "next/server";

import { db } from "@/src/server/db/drizzle";
import { createDidWeb, CustomDIDDocument } from "@/src/server/did";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { Params, paramsSchema } from "./types";

type CustomDocument = DIDDocument & CustomDIDDocument;

export async function GET(request: NextRequest, { params }: Params) {
  const { id } = paramsSchema.parse(params);

  const did = createDidWeb(id);

  // Fetch profile from database
  const userProfile = await db.query.user.findFirst({
    columns: {
      address: true,
      id: true,
      username: true,
    },
    where: (row, { eq }) => eq(row.did, did),
    with: {
      profile: {
        columns: {
          backgroundKey: true,
          bio: true,
          imageKey: true,
        },
      },
    },
  });

  const image = userProfile?.profile?.imageKey
    ? cdnURL(S3Path.profile(userProfile.id).image(userProfile.profile.imageKey))
    : undefined;
  const background = userProfile?.profile?.backgroundKey
    ? cdnURL(
        S3Path.profile(userProfile.id).background(
          userProfile.profile.backgroundKey
        )
      )
    : undefined;

  const doc: CustomDocument = {
    "@context": "https://www.w3.org/did/v1",
    id: did,
    profile: {
      address: userProfile?.address ?? undefined,
      background,
      bio: userProfile?.profile?.bio ?? undefined,
      image,
      username: userProfile?.username,
    },
  };

  return NextResponse.json(doc);
}
