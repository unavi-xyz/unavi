import { WorldMetadata } from "@wired-protocol/types";
import Link from "next/link";

import PlayerCount from "@/app/(navbar)/explore/PlayerCount";

import { SpaceId } from "../utils/parseSpaceId";
import { toHex } from "../utils/toHex";
import { CardImage } from "./Card";
import Tooltip from "./Tooltip";

interface Props {
  id: SpaceId;
  metadata: WorldMetadata;
  tokenId?: number;
  sizes?: string;
}

/**
 * Wrapper around {@link Card} that links to the space page, and shows the player count.
 */
export default function SpaceCard({ id, metadata, tokenId, sizes }: Props) {
  return (
    <div>
      <div className="group relative">
        <Link
          href={id.type === "id" ? `/space/${id.value}` : `/space/${toHex(id.value)}`}
          className="rounded-3xl"
        >
          <CardImage group image={metadata.info?.image} sizes={sizes}>
            <PlayerCount metadata={metadata} />
          </CardImage>

          <div className="absolute bottom-0 z-10 h-full w-full rounded-b-3xl bg-gradient-to-t from-black/30 via-transparent to-transparent opacity-0 transition duration-100 ease-out group-hover:scale-105 group-hover:opacity-100" />
        </Link>

        <div className="absolute bottom-0 left-0 z-20 hidden animate-fadeIn pb-4 pl-3 group-hover:block">
          <Link
            href={id.type === "id" ? `/play?id=${id.value}` : `/play?tokenId=${toHex(id.value)}`}
            className="rounded-xl bg-white px-4 py-1.5 text-xl font-bold shadow transition hover:bg-neutral-200 hover:shadow-md active:bg-neutral-300"
          >
            Play
          </Link>
        </div>
      </div>

      <div className="space-x-2 pb-1 pt-2.5">
        {tokenId !== undefined ? (
          <Tooltip text="Space is published to the blockchain as an NFT" delayDuration={200}>
            <span className="w-fit cursor-default rounded-full border border-sky-700/40 bg-sky-100 px-2 text-sm text-sky-700">
              NFT
            </span>
          </Tooltip>
        ) : null}

        <span className="text-xl font-bold text-neutral-900">{metadata.info?.name}</span>
      </div>
    </div>
  );
}
