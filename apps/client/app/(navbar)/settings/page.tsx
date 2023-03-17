import { Metadata as NextMetadata } from "next";

import { fetchProfileFromAddress } from "../../../src/server/helpers/fetchProfileFromAddress";
import { getServerSession } from "../../../src/server/helpers/getServerSession";
import { metadata as baseMetadata } from "../../layout";
import RainbowkitWrapper from "../RainbowkitWrapper";
import SessionProvider from "../SessionProvider";
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
  const session = await getServerSession();

  if (!session || !session.address) return <div className="pt-10 text-center">Not signed in.</div>;

  const profile = await fetchProfileFromAddress(session.address);

  return (
    <SessionProvider>
      <RainbowkitWrapper>
        <div className="flex w-full justify-center">
          <div className="mx-4 w-full max-w-lg space-y-4 py-8">
            <div className="text-center text-3xl font-black">Settings</div>
            <Handle profile={profile} />
            {profile && <Metadata profile={profile} />}
          </div>
        </div>
      </RainbowkitWrapper>
    </SessionProvider>
  );
}
