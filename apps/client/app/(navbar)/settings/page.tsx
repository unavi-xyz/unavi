import { Metadata as NextMetadata } from "next";

import AuthProvider from "@/src/client/AuthProvider";
import { env } from "@/src/env.mjs";
import { getUserSession } from "@/src/server/auth/getUserSession";
import { fetchProfileFromAddress } from "@/src/server/helpers/fetchProfileFromAddress";

import { metadata as baseMetadata } from "../../layout";
import RainbowkitWrapper from "../RainbowkitWrapper";
import Handle from "./Handle";
import Metadata from "./Metadata";

const TITLE = "Settings";

export const metadata: NextMetadata = {
  title: TITLE,
  openGraph: {
    ...baseMetadata.openGraph,
    title: TITLE,
  },
  twitter: {
    ...baseMetadata.twitter,
    title: TITLE,
  },
};

export default async function Settings() {
  const session = await getUserSession();

  if (!session || !session.user) return <div className="pt-10 text-center">Not signed in.</div>;

  const profile = await fetchProfileFromAddress(session.user.address);

  return (
    <AuthProvider>
      <RainbowkitWrapper>
        <div className="flex w-full justify-center">
          <div className="mx-4 w-full max-w-lg space-y-4 py-8">
            <div className="text-center text-3xl font-black">Settings</div>
            <Handle profile={profile} />
            {profile && env.NEXT_PUBLIC_HAS_S3 ? <Metadata profile={profile} /> : null}
          </div>
        </div>
      </RainbowkitWrapper>
    </AuthProvider>
  );
}
