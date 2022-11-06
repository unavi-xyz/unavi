import { useGetPublicationQuery } from "@wired-labs/lens";
import Image from "next/image";
import { useEffect, useState } from "react";

import Button from "../../ui/Button";
import { getMediaURL } from "../../utils/getMediaURL";
import { isFromCDN } from "../../utils/isFromCDN";

interface Props {
  spaceId: string;
  loaded: boolean;
}

export default function LoadingScreen({ spaceId, loaded }: Props) {
  const [entered, setEntered] = useState(false);
  const [enterTransitionFinished, setEnterTransitionFinished] = useState(false);

  const [{ data }] = useGetPublicationQuery({
    variables: { request: { publicationId: spaceId } },
    pause: !spaceId,
  });

  useEffect(() => {
    if (!entered) return;

    const timeout = setTimeout(() => {
      setEnterTransitionFinished(true);
    }, 700);

    return () => clearTimeout(timeout);
  }, [entered]);

  if (enterTransitionFinished) return null;

  const transitionClass = entered
    ? "opacity-0 backdrop-blur-0"
    : "opacity-100 backdrop-blur-xl";

  const image = getMediaURL(data?.publication?.metadata.media[0]);

  return (
    <div
      className={`absolute z-50 h-screen w-screen bg-surface pb-8 transition duration-700 ${transitionClass}`}
    >
      <div className="flex h-full flex-col items-center justify-center">
        {data && (
          <div className="max-w-content space-y-6">
            <div className="flex w-full min-w-fit flex-col justify-between">
              <div className="space-y-4">
                <div className="flex justify-center text-3xl font-black">
                  {data?.publication?.metadata.name}
                </div>

                <div className="mx-auto w-1/2">
                  <div className="aspect-card h-full w-full rounded-3xl bg-primaryContainer">
                    <div className="relative h-full w-full object-cover">
                      {image &&
                        (isFromCDN(image) ? (
                          <Image
                            src={image}
                            priority
                            fill
                            sizes="30vw"
                            alt="space preview"
                            className="rounded-3xl object-cover"
                          />
                        ) : (
                          // eslint-disable-next-line @next/next/no-img-element
                          <img
                            src={image}
                            alt="space preview"
                            className="h-full w-full rounded-3xl object-cover"
                            crossOrigin="anonymous"
                          />
                        ))}
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div className="flex w-full justify-center">
              <Button
                variant="filled"
                loading={!loaded}
                onClick={() => setEntered(true)}
              >
                <div className="px-8 py-1 text-2xl">Enter</div>
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
