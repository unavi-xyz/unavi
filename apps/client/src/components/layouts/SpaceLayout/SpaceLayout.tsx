import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";

import { useMediaImage } from "../../../helpers/lens/hooks/useMediaImage";
import { useLensStore } from "../../../helpers/lens/store";
import Button from "../../base/Button";
import SpaceTab from "./SpaceTab";
import { SpaceLayoutProps } from "./getSpaceLayoutProps";

interface Props extends SpaceLayoutProps {
  children: React.ReactNode;
}

export default function SpaceLayout({
  children,
  metadata,
  publication,
}: Props) {
  const router = useRouter();
  const id = router.query.id as string;

  const media = publication?.metadata.media[0];
  const image = useMediaImage(media);

  const handle = useLensStore((state) => state.handle);
  const isAuthor = handle && handle === publication?.profile.handle;

  return (
    <>
      <Head>
        <title>{metadata.title} / The Wired</title>
        <meta name="description" content={metadata.description} />

        <meta itemProp="name" content={metadata.title} />
        <meta itemProp="description" content={metadata.description} />
        <meta itemProp="image" content={metadata.image} />

        <meta property="og:title" content={metadata.title} />
        <meta property="og:description" content={metadata.description} />
        <meta property="og:image" content={metadata.image} />
        <meta property="og:image:width" content={"595.2px"} />
        <meta property="og:image:height" content={"357.11px"} />

        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:title" content={metadata.title} />
        <meta name="twitter:description" content={metadata.description} />
        <meta name="twitter:image" content={metadata.image} />
      </Head>

      <div className="mx-8 h-full">
        <div className="max-w mx-auto py-8 w-full h-full space-y-8">
          <div className="flex space-x-8">
            <div className="w-full rounded-3xl aspect-card bg-secondaryContainer">
              {image && (
                <img
                  src={image}
                  alt="space preview"
                  className="object-cover rounded-3xl w-full h-full"
                />
              )}
            </div>

            <div className="w-2/3 min-w-fit flex flex-col justify-between space-y-8">
              <div className="space-y-4">
                <div className="font-black text-3xl flex justify-center">
                  {publication?.metadata.name}
                </div>

                <div className="space-y-2">
                  <div className="font-bold flex space-x-1">
                    <div>By</div>
                    <Link href={`/user/${publication?.profile.handle}`}>
                      <div className="hover:underline cursor-pointer">
                        @{publication?.profile.handle}
                      </div>
                    </Link>
                  </div>
                </div>
              </div>

              <Link href={`/app/${id}`} passHref>
                <div>
                  <Button variant="filled" fullWidth>
                    <div className="py-2">Join Space</div>
                  </Button>
                </div>
              </Link>
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex space-x-4">
              <SpaceTab href={`/space/${id}`} text="About" />

              {isAuthor && (
                <SpaceTab href={`/space/${id}/settings`} text="Settings" />
              )}
            </div>

            <div>{children}</div>
          </div>
        </div>
      </div>
    </>
  );
}
