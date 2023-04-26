import { WorldMetadata } from "@wired-protocol/types";

import AuthProvider from "@/src/client/AuthProvider";
import { getUserSession } from "@/src/server/auth/getUserSession";
import { SpaceId } from "@/src/utils/parseSpaceId";

import RainbowkitWrapper from "../../RainbowkitWrapper";
import Delete from "./Delete";
import Mint from "./Mint";

interface Props {
  id: SpaceId;
  metadata: WorldMetadata;
}

export default async function Settings({ id, metadata }: Props) {
  const session = await getUserSession();
  if (!session?.user.address) return null;

  return (
    <AuthProvider>
      <RainbowkitWrapper>
        <div className="space-y-8">
          {id.type === "id" ? <Mint id={id} metadata={metadata} /> : null}
          <Delete id={id} address={session.user.address} />
        </div>
      </RainbowkitWrapper>
    </AuthProvider>
  );
}
