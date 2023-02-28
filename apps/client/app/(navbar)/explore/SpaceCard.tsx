import { ERC721Metadata } from "contracts";
import Link from "next/link";
import { Suspense } from "react";

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
        <Suspense fallback={null}>
          {/* @ts-expect-error Server Component */}
          <PlayerCount id={id} />
        </Suspense>
      </Card>
    </Link>
  );
}
