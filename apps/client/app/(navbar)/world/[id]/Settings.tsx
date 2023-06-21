import { WorldId } from "@/src/utils/parseWorldId";

import Delete from "./Delete";

interface Props {
  id: WorldId;
}

export default async function Settings({ id }: Props) {
  return <Delete id={id} />;
}
