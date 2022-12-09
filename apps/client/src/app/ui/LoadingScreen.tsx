import Image from "next/image";
import { useEffect, useState } from "react";

import LoadingBar from "../../ui/LoadingBar";
import { isFromCDN } from "../../utils/isFromCDN";

interface Props {
  text?: string | null;
  image?: string | null;
  loaded: boolean;
  loadingProgress: number;
  loadingText: string;
}

export default function LoadingScreen({
  text,
  image,
  loaded,
  loadingProgress,
  loadingText,
}: Props) {
  const [entered, setEntered] = useState(false);
  const [enterTransitionFinished, setEnterTransitionFinished] = useState(false);

  useEffect(() => {
    if (!loaded) return;
    setEntered(true);
  }, [loaded]);

  useEffect(() => {
    if (!entered) return;

    const timeout = setTimeout(() => {
      setEnterTransitionFinished(true);
    }, 500);

    return () => clearTimeout(timeout);
  }, [entered]);

  if (enterTransitionFinished) return null;

  const transitionClass = entered
    ? "opacity-0 backdrop-blur-0"
    : "opacity-100 backdrop-blur-xl";

  return (
    <div
      className={`absolute z-50 h-screen w-screen bg-white pb-8 transition duration-500 ${transitionClass}`}
    >
      <div className="flex h-full animate-fadeInSlow flex-col items-center justify-center">
        <div className="max-w-content space-y-6">
          <div className="flex w-full min-w-fit flex-col justify-between">
            <div className="space-y-4">
              <div className="flex justify-center text-3xl font-black">
                {text}
              </div>

              <div className="mx-auto px-8 md:w-1/2">
                <div className="aspect-card h-full w-full rounded-3xl bg-sky-100">
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
            <LoadingBar progress={loadingProgress} text={loadingText} />
          </div>
        </div>
      </div>
    </div>
  );
}
