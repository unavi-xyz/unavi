import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/router";

import { useLens } from "../../../client/lens/hooks/useLens";
import { PublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { trimHandle } from "../../../client/lens/utils/trimHandle";
import { trpc } from "../../../client/trpc";
import { env } from "../../../env/client.mjs";
import Button from "../../../ui/Button";
import NavigationTab from "../../../ui/NavigationTab";
import { isFromCDN } from "../../../utils/isFromCDN";
import MetaTags from "../../MetaTags";

const host =
  process.env.NODE_ENV === "development" ? "localhost:4000" : env.NEXT_PUBLIC_DEFAULT_HOST;

export interface Props extends PublicationProps {
  children: React.ReactNode;
}

export default function SpaceLayout({ children, metadata, publication, image }: Props) {
  const router = useRouter();
  const id = router.query.id as string;

  const { handle } = useLens();
  const author = trimHandle(publication?.profile.handle);
  const isAuthor = handle && handle === author;

  const { data: playerCount } = trpc.public.playerCount.useQuery({ id });

  return (
    <>
      <MetaTags
        title={metadata.title ?? id}
        description={metadata.description ?? undefined}
        image={metadata.image ?? undefined}
        card="summary_large_image"
      />

      <div className="mx-4 h-full">
        <div className="max-w-content mx-auto h-full w-full space-y-8 py-8">
          <div className="flex flex-col space-y-8 md:flex-row md:space-y-0 md:space-x-8">
            <div className="aspect-card h-full w-full rounded-2xl bg-sky-100">
              <div className="relative h-full w-full object-cover">
                {image &&
                  (isFromCDN(image) ? (
                    <Image
                      src={image}
                      priority
                      fill
                      sizes="40vw"
                      alt=""
                      className="rounded-2xl object-cover"
                    />
                  ) : (
                    <img
                      src={image}
                      alt=""
                      className="h-full w-full rounded-2xl object-cover"
                      crossOrigin="anonymous"
                    />
                  ))}
              </div>
            </div>

            <div className="flex flex-col justify-between space-y-8 md:w-2/3">
              <div className="space-y-4">
                <div className="text-center text-3xl font-black">{publication?.metadata.name}</div>

                <div className="space-y-2">
                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-neutral-500">By</div>
                    <Link href={`/user/${author}`}>
                      <div className="cursor-pointer hover:underline">@{author}</div>
                    </Link>
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

              <Link href={`/app/${id}`}>
                <div>
                  <Button variant="filled" fullWidth>
                    <div className="py-2">Join Space</div>
                  </Button>
                </div>
              </Link>
            </div>
          </div>

          <div className="space-y-4 pb-4">
            <div className="flex space-x-4">
              <NavigationTab href={`/space/${id}`} text="About" />

              {isAuthor && <NavigationTab href={`/space/${id}/settings`} text="Settings" />}
            </div>

            <div>{children}</div>
          </div>
        </div>
      </div>
    </>
  );
}
