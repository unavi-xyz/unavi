import Link from "next/link";
import { useRouter } from "next/router";

import { LocalSpace } from "../../helpers/indexedDB/localSpaces/types";
import Card from "../base/Card";

interface Props {
  localSpace: LocalSpace;
}

export default function LocalSpaceCard({ localSpace }: Props) {
  const router = useRouter();

  return (
    <Link href={`/create/${localSpace.id}`} passHref>
      <div className="h-96">
        <Card text={localSpace.name} image={localSpace.image} />
      </div>
    </Link>
  );
}
