import { getServerSession } from "@/src/server/helpers/getServerSession";

import RainbowkitWrapper from "../../RainbowkitWrapper";
import SessionProvider from "../../SessionProvider";
import Delete from "./Delete";

interface Props {
  id: string;
}

export default async function Settings({ id }: Props) {
  const session = await getServerSession();
  if (!session?.address) return null;

  return (
    <SessionProvider>
      <RainbowkitWrapper>
        <div className="space-y-12">
          <Delete id={id} address={session.address} />
        </div>
      </RainbowkitWrapper>
    </SessionProvider>
  );
}
