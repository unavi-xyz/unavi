import { useGetPublicationQuery } from "lens";
import Image from "next/image";
import { useEffect, useState } from "react";

import Spinner from "../../ui/Spinner";
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
    if (!loaded) return;
    setEntered(true);
  }, [loaded]);

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
                            alt=""
                            className="rounded-3xl object-cover"
                          />
                        ) : (
                          <img
                            src={image}
                            alt=""
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
              {!loaded && <Spinner />}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
