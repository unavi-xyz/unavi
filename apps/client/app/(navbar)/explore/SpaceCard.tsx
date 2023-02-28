import { ERC721Metadata } from "contracts";
import Link from "next/link";

import Card from "../../../src/ui/Card";
import { numberToHexDisplay } from "../../../src/utils/numberToHexDisplay";
import PlayerCount from "./PlayerCount";

interface Props {
  id: number;
  metadata: ERC721Metadata;
  sizes?: string;
}

export default function SpaceCard({ id, metadata, sizes }: Props) {
  return (
    <Link href={`/space/${numberToHexDisplay(id)}`} className="rounded-xl">
      <Card text={metadata?.name} image={metadata?.image} sizes={sizes} animateEnter>
        <PlayerCount id={id} />
      </Card>
    </Link>
  );
}
