import { ERC721Metadata } from "contracts";
import Image from "next/image";
import Link from "next/link";

import { useSession } from "../../../client/auth/useSession";
import { trpc } from "../../../client/trpc";
import { env } from "../../../env/client.mjs";
import { Profile } from "../../../server/helpers/getProfileFromAddress";
import NavigationTab from "../../../ui/NavigationTab";
import { isFromCDN } from "../../../utils/isFromCDN";
import { numberToHexDisplay } from "../../../utils/numberToHexDisplay";
import MetaTags from "../../MetaTags";

const host =
  process.env.NODE_ENV === "development" ? "localhost:4000" : env.NEXT_PUBLIC_DEFAULT_HOST;

export interface Props {
  id: number;
  author: Profile | null;
  metadata: ERC721Metadata | null;
  children: React.ReactNode;
}

export default function SpaceLayout({ id, author, metadata, children }: Props) {
  const { data: playerCount } = trpc.public.playerCount.useQuery({ id });

  const { data: session } = useSession();
  const isAuthor = session && session.address === author?.owner;

  const hexId = numberToHexDisplay(id);

  return (
    <>
      <MetaTags
        title={metadata?.name ?? `Space ${hexId}`}
        description={metadata?.description ?? ""}
        image={metadata?.image}
        card="summary_large_image"
      />

      <div className="mx-4 h-full">
        <div className="max-w-content mx-auto h-full w-full space-y-8 py-8">
          <div className="flex flex-col space-y-8 md:flex-row md:space-y-0 md:space-x-8">
            <div className="aspect-card h-full w-full rounded-2xl bg-neutral-200">
              <div className="relative h-full w-full object-cover">
                {metadata?.image &&
                  (isFromCDN(metadata.image) ? (
                    <Image
                      src={metadata.image}
                      priority
                      fill
                      sizes="40vw"
                      alt=""
                      className="rounded-2xl object-cover"
                    />
                  ) : (
                    <img
                      src={metadata.image}
                      alt=""
                      className="h-full w-full rounded-2xl object-cover"
                      crossOrigin="anonymous"
                    />
                  ))}
              </div>
            </div>

            <div className="flex flex-col justify-between space-y-8 md:w-2/3">
              <div className="space-y-4">
                <div className="text-center text-3xl font-black">{metadata?.name}</div>

                <div className="space-y-1">
                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-neutral-500">By</div>

                    {author && (
                      <Link href={`/user/${numberToHexDisplay(author.id)}`}>
                        <div className="cursor-pointer decoration-2 hover:underline">
                          {author.handle?.string ?? author.owner}
                        </div>
                      </Link>
                    )}
                  </div>

                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-neutral-500">At</div>
                    <div>{host}</div>
                  </div>

                  {playerCount && playerCount > 0 ? (
                    <div className="flex justify-center space-x-1 font-bold md:justify-start">
                      <div>{playerCount}</div>
                      <div className="text-neutral-500">
                        connected player{playerCount === 1 ? null : "s"}
                      </div>
                    </div>
                  ) : null}
                </div>
              </div>

              <Link
                href={`/play/${hexId}`}
                className="rounded-full bg-neutral-900 py-3 text-center text-lg font-bold text-white transition hover:scale-105 active:opacity-90"
              >
                Join Space
              </Link>
            </div>
          </div>

          <div className="space-y-4 pb-4">
            <div className="flex space-x-4">
              <NavigationTab href={`/space/${hexId}`} text="About" />

              {isAuthor && <NavigationTab href={`/space/${hexId}/settings`} text="Settings" />}
            </div>

            <div>{children}</div>
          </div>
        </div>
      </div>
    </>
  );
}
