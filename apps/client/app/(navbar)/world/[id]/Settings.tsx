import { WorldMetadata } from "@wired-protocol/types";

import { getUserSession } from "@/src/server/auth/getUserSession";
import { WorldId } from "@/src/utils/parseWorldId";

import Delete from "./Delete";

interface Props {
  id: WorldId;
  metadata: WorldMetadata;
}

export default async function Settings({ id, metadata }: Props) {
  const session = await getUserSession();
  if (!session?.user.address) return null;

  return <Delete id={id} />;
}
