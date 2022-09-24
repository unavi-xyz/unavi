import Image from "next/future/image";
import Link from "next/link";
import { useRouter } from "next/router";

import { useLens } from "../../../lib/lens/hooks/useLens";
import { trimHandle } from "../../../lib/lens/utils/trimHandle";
import Button from "../../../ui/base/Button";
import NavigationTab from "../../../ui/base/NavigationTab";
import MetaTags from "../../../ui/MetaTags";
import { SpaceLayoutProps } from "./getSpaceLayoutProps";

export default function SpaceLayout({
  children,
  metadata,
  publication,
  playerCount,
  image,
  host,
}: SpaceLayoutProps) {
  const router = useRouter();
  const id = router.query.id as string;

  const { handle } = useLens();
  const author = trimHandle(publication?.profile.handle);
  const isAuthor = handle && handle === author;

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
            <div className="aspect-card h-full w-full rounded-3xl bg-primaryContainer">
              <div className="relative h-full w-full object-cover">
                {image && (
                  <Image
                    src={image}
                    priority
                    fill
                    sizes="40vw"
                    alt="space preview"
                    className="rounded-3xl"
                  />
                )}
              </div>
            </div>

            <div className="flex min-w-fit flex-col justify-between space-y-8 md:w-2/3">
              <div className="space-y-4">
                <div className="flex justify-center text-3xl font-black">
                  {publication?.metadata.name}
                </div>

                <div className="space-y-2">
                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-outline">By</div>
                    <Link href={`/user/${author}`}>
                      <div className="cursor-pointer hover:underline">
                        @{author}
                      </div>
                    </Link>
                  </div>

                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-outline">At</div>
                    <div>{host}</div>
                  </div>

                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-outline">With</div>
                    <div>{playerCount ?? "??"}</div>
                    <div className="text-outline">
                      connected player{playerCount === 1 ? null : "s"}
                    </div>
                  </div>
                </div>
              </div>

              <Link href={`/app/${id}`} passHref>
                <a>
                  <Button variant="filled" fullWidth>
                    <div className="py-2">Join Space</div>
                  </Button>
                </a>
              </Link>
            </div>
          </div>

          <div className="space-y-4 pb-4">
            <div className="flex space-x-4">
              <NavigationTab href={`/space/${id}`} text="About" />

              {isAuthor && (
                <NavigationTab href={`/space/${id}/settings`} text="Settings" />
              )}
            </div>

            <div>{children}</div>
          </div>
        </div>
      </div>
    </>
  );
}
