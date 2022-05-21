import Link from "next/link";

import { LocalSpace } from "../../helpers/indexeddb/LocalSpace/types";
import Card from "../base/Card";

interface Props {
  localSpace: LocalSpace;
}

export default function LocalSpaceCard({ localSpace }: Props) {
  const image = localSpace.image ?? localSpace.generatedImage;
  const imageUrl = image ? URL.createObjectURL(image) : undefined;

  return (
    <Link href={`/create/${localSpace.id}`} passHref>
      <div className="aspect-card">
        <Card text={localSpace.name} image={imageUrl} />
      </div>
    </Link>
  );
}
