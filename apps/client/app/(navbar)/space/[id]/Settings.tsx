import { fetchSpaceMetadata } from "@/src/server/helpers/fetchSpaceMetadata";
import { fetchSpaceOwner } from "@/src/server/helpers/fetchSpaceOwner";
import { getServerSession } from "@/src/server/helpers/getServerSession";

import RainbowkitWrapper from "../../RainbowkitWrapper";
import SessionProvider from "../../SessionProvider";
import Delete from "./Delete";
import Host from "./Host";

interface Props {
  id: number;
}

export default async function Settings({ id }: Props) {
  const [session, owner, metadata] = await Promise.all([
    getServerSession(),
    fetchSpaceOwner(id),
    fetchSpaceMetadata(id),
  ]);

  if (!session?.address) return null;

  const isOwner = session.address === owner;

  if (!isOwner) return null;

  return (
    <SessionProvider>
      <RainbowkitWrapper>
        <div className="space-y-12">
          <Host id={id} metadata={metadata} />
          <Delete id={id} address={session.address} />
        </div>
      </RainbowkitWrapper>
    </SessionProvider>
  );
}
