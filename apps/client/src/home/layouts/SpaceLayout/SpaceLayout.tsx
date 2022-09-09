import Image from "next/future/image";
import Link from "next/link";
import { useRouter } from "next/router";

import { useLens } from "../../../lib/lens/hooks/useLens";
import { trimHandle } from "../../../lib/lens/utils/trimHandle";
import MetaTags from "../../../ui/MetaTags";
import Button from "../../../ui/base/Button";
import NavigationTab from "../../../ui/base/NavigationTab";
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
        title={metadata.title}
        description={metadata.description}
        image={metadata.image}
        card="summary_large_image"
      />

      <div className="mx-4 h-full">
        <div className="max-w-content mx-auto py-8 w-full h-full space-y-8">
          <div className="flex flex-col md:flex-row space-y-8 md:space-y-0 md:space-x-8">
            <div className="w-full h-full rounded-3xl aspect-card bg-primaryContainer">
              <div className="relative object-cover w-full h-full">
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

            <div className="md:w-2/3 min-w-fit flex flex-col justify-between space-y-8">
              <div className="space-y-4">
                <div className="font-black text-3xl flex justify-center">
                  {publication?.metadata.name}
                </div>

                <div className="space-y-2">
                  <div className="font-bold flex space-x-1 justify-center md:justify-start">
                    <div className="text-outline">By</div>
                    <Link href={`/user/${author}`}>
                      <a className="hover:underline cursor-pointer">
                        @{author}
                      </a>
                    </Link>
                  </div>

                  <div className="font-bold flex space-x-1 justify-center md:justify-start">
                    <div className="text-outline">At</div>
                    <div>{host}</div>
                  </div>

                  <div className="font-bold flex space-x-1 justify-center md:justify-start">
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
