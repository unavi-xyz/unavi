import { ERC721Metadata } from "contracts";
import Link from "next/link";

import PlayerCount from "../../app/(navbar)/explore/PlayerCount";
import { toHex } from "../utils/toHex";
import { CardImage } from "./Card";
import { CardText } from "./Card";

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
    <div>
      <div className="group relative">
        <Link href={`/space/${toHex(id)}`} className="rounded-[2rem]">
          <CardImage group image={metadata.image} sizes={sizes}>
            <PlayerCount id={id} />
          </CardImage>

          <div className="absolute bottom-0 z-10 h-full w-full rounded-b-[2rem] bg-gradient-to-t from-black/30 via-transparent to-transparent opacity-0 transition duration-100 ease-out group-hover:scale-105 group-hover:opacity-100" />
        </Link>

        <div className="absolute bottom-0 left-0 z-20 hidden animate-fadeIn pl-4 pb-4 group-hover:block">
          <Link
            href={`/play/${toHex(id)}`}
            className="rounded-xl bg-neutral-900 px-4 py-1.5 text-xl font-bold text-white shadow transition hover:bg-neutral-700 hover:shadow-md active:bg-neutral-800"
          >
            Enter
          </Link>
        </div>
      </div>

      <CardText text={metadata.name} />
    </div>
  );
}
