import Link from "next/link";

import PlayerCount from "@/app/(navbar)/explore/PlayerCount";

import { SpaceMetadata } from "../server/helpers/readSpaceMetadata";
import { toHex } from "../utils/toHex";
import { CardImage } from "./Card";
import { CardText } from "./Card";

interface Props {
  id: number;
  metadata: SpaceMetadata;
  sizes?: string;
}

/**
 * Wrapper around {@link Card} that links to the space page, and shows the player count.
 */
export default function SpaceCard({ id, metadata, sizes }: Props) {
  return (
    <div>
      <div className="group relative">
        <Link href={`/space/${toHex(id)}`} className="rounded-3xl">
          <CardImage group image={metadata.image} sizes={sizes}>
            <PlayerCount metadata={metadata} />
          </CardImage>

          <div className="absolute bottom-0 z-10 h-full w-full rounded-b-3xl bg-gradient-to-t from-black/30 via-transparent to-transparent opacity-0 transition duration-100 ease-out group-hover:scale-105 group-hover:opacity-100" />
        </Link>

        <div className="absolute bottom-0 left-0 z-20 hidden animate-fadeIn pl-3 pb-4 group-hover:block">
          <Link
            href={`/play?space=nft://${toHex(id)}`}
            className="rounded-xl bg-white px-4 py-1.5 text-xl font-bold shadow transition hover:bg-neutral-200 hover:shadow-md active:bg-neutral-300"
          >
            Play
          </Link>
        </div>
      </div>

      <CardText text={metadata.title} />
    </div>
  );
}
