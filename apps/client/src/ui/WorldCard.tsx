import { WorldMetadata } from "@wired-protocol/types";
import Link from "next/link";

import PlayerCount from "@/app/(navbar)/PlayerCount";

import { env } from "../env.mjs";
import { CardImage } from "./Card";

interface Props {
  id: string;
  uri: string;
  metadata: WorldMetadata;
  tokenId?: number;
  sizes?: string;
}

/**
 * Wrapper around {@link Card} that links to the world page, and shows the player count.
 */
export default function WorldCard({
  id,
  uri,
  metadata,

  sizes,
}: Props) {
  return (
    <div>
      <div className="group relative">
        <Link href={`/world/${id}`} className="rounded-3xl">
          <CardImage group image={metadata.info?.image} sizes={sizes}>
            <PlayerCount
              uri={uri}
              host={metadata.info?.host || env.NEXT_PUBLIC_DEFAULT_HOST}
            />
          </CardImage>

          <div className="absolute bottom-0 z-10 h-full w-full rounded-b-3xl bg-gradient-to-t from-black/30 via-transparent to-transparent opacity-0 transition duration-100 ease-out group-hover:scale-105 group-hover:opacity-100" />
        </Link>

        <div className="absolute bottom-0 left-0 z-20 mb-4 ml-3 hidden group-hover:block">
          <Link
            href={`/play?id=${id}`}
            className="rounded-xl bg-white px-4 py-1.5 text-xl font-bold shadow transition hover:bg-neutral-200 hover:shadow-md active:bg-neutral-300"
          >
            Play
          </Link>
        </div>
      </div>

      <div className="space-x-2 pb-1 pt-2.5">
        <span className="text-xl font-bold text-neutral-900">
          {metadata.info?.name}
        </span>
      </div>
    </div>
  );
}
