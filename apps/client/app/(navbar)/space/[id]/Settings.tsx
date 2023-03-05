import { fetchSpace } from "../../../../src/server/helpers/fetchSpace";
import { getServerSession } from "../../../../src/server/helpers/getServerSession";
import RainbowkitWrapper from "../../RainbowkitWrapper";
import SessionProvider from "../../SessionProvider";
import Delete from "./Delete";

type Params = { id: string };

export default async function Settings({ params: { id } }: { params: Params }) {
  const spaceId = parseInt(id);
  const [session, space] = await Promise.all([getServerSession(), fetchSpace(spaceId)]);

  if (!space || !session?.address) return null;

  const isOwner = session.address === space.owner;

  if (!isOwner) return null;

  return (
    <SessionProvider>
      <RainbowkitWrapper>
        <div className="space-y-12">
          <Delete id={spaceId} address={session.address} />
        </div>
      </RainbowkitWrapper>
    </SessionProvider>
  );
}
