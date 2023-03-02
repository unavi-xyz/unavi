import { SessionProvider } from "next-auth/react";

import { fetchProfileFromAddress } from "../../../src/server/helpers/fetchProfileFromAddress";
import { getServerSession } from "../../../src/server/helpers/getServerSession";
import RainbowkitWrapper from "../RainbowkitWrapper";
import Handle from "./Handle";
import Metadata from "./Metadata";

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
