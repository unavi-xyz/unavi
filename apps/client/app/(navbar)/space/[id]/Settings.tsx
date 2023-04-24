import { WorldMetadata } from "@wired-protocol/types";

import { getServerSession } from "@/src/server/helpers/getServerSession";
import { SpaceId } from "@/src/utils/parseSpaceId";

import RainbowkitWrapper from "../../RainbowkitWrapper";
import SessionProvider from "../../SessionProvider";
import Delete from "./Delete";
import Mint from "./Mint";

interface Props {
  id: SpaceId;
  metadata: WorldMetadata;
}

export default async function Settings({ id, metadata }: Props) {
  const session = await getServerSession();
  if (!session?.address) return null;

  return (
    <SessionProvider>
      <RainbowkitWrapper>
        <div className="space-y-8">
          {id.type === "id" ? <Mint id={id} metadata={metadata} /> : null}
          <Delete id={id} address={session.address} />
        </div>
      </RainbowkitWrapper>
    </SessionProvider>
  );
}
