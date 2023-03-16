import { ERC721Metadata } from "contracts";
import Link from "next/link";

import PlayerCount from "../../app/(navbar)/explore/PlayerCount";
import { toHex } from "../utils/toHex";
import Card from "./Card";

interface Props {
  id: number;
  metadata: ERC721Metadata;
  sizes?: string;
}

/**
 * Wrapper around {@link Card} that links to the space page, and shows the player count.
 * @param id Space ID
 * @param metadata Space metadata
 */
export default function SpaceCard({ id, metadata, sizes }: Props) {
  return (
    <Link href={`/space/${toHex(id)}`} className="rounded-xl">
      <Card text={metadata.name} image={metadata.image} sizes={sizes}>
        <PlayerCount id={id} />
      </Card>
    </Link>
  );
}
