import Link from "next/link";
import { useRouter } from "next/router";

import { PublicationProps } from "../../../helpers/lens/getPublicationProps";
import { useMediaImage } from "../../../helpers/lens/hooks/useMediaImage";
import { useLensStore } from "../../../helpers/lens/store";
import { trimHandle } from "../../../helpers/lens/utils";
import Button from "../../base/Button";
import NavigationTab from "../../base/NavigationTab";
import MetaTags from "../../ui/MetaTags";

interface Props extends PublicationProps {
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
  const author = trimHandle(publication?.profile.handle);
  const isAuthor = handle && handle === author;

  return (
    <>
      <MetaTags
        title={metadata.title}
        description={metadata.description}
        image={metadata.image}
        imageWidth="595.2px"
        imageHeight="357.11px"
        card="summary_large_image"
      />

      <div className="mx-4 h-full">
        <div className="max-w mx-auto py-8 w-full h-full space-y-8">
          <div className="flex flex-col md:flex-row space-y-8 md:space-y-0 md:space-x-8">
            <div className="w-full rounded-3xl aspect-card bg-secondaryContainer">
              {image && (
                <img
                  src={image}
                  alt="space preview"
                  className="object-cover rounded-3xl w-full h-full"
                />
              )}
            </div>

            <div className="md:w-2/3 min-w-fit flex flex-col justify-between space-y-8">
              <div className="space-y-4">
                <div className="font-black text-3xl flex justify-center">
                  {publication?.metadata.name}
                </div>

                <div className="space-y-2">
                  <div className="font-bold flex space-x-1 justify-center md:justify-start">
                    <div>By</div>
                    <Link href={`/user/${author}`}>
                      <a className="hover:underline cursor-pointer">
                        @{author}
                      </a>
                    </Link>
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
