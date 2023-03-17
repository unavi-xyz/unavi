import Image from "next/image";
import { useEffect, useState } from "react";

import LoadingBar from "../../ui/LoadingBar";
import { isFromCDN } from "../../utils/isFromCDN";
import { usePlayStore } from "../store";

interface Props {
  text?: string | null;
  image?: string | null;
  loadingProgress: number;
  loadingText: string;
}

export default function LoadingScreen({ text, image, loadingProgress, loadingText }: Props) {
  const [entered, setEntered] = useState(false);
  const [enterTransitionFinished, setEnterTransitionFinished] = useState(false);

  const errorLoading = usePlayStore((state) => state.errorLoading);

  useEffect(() => {
    if (loadingProgress === 1) setEntered(true);
    else setEntered(false);
  }, [loadingProgress]);

  useEffect(() => {
    if (!entered) return;

    const timeout = setTimeout(() => {
      setEnterTransitionFinished(true);
    }, 500);

    return () => clearTimeout(timeout);
  }, [entered]);

  if (enterTransitionFinished) return null;

  const transitionClass = entered ? "opacity-0 backdrop-blur-0" : "opacity-100 backdrop-blur-3xl";

  return (
    <div
      className={`absolute z-50 h-screen w-screen bg-white/50 pb-8 transition duration-500 ${transitionClass}`}
    >
      <div className="flex h-full animate-floatInSlow flex-col items-center justify-center">
        <div className="max-w-content space-y-6">
          <div className="flex w-full min-w-fit flex-col justify-between">
            <div className="space-y-4">
              <div className="flex justify-center text-3xl font-black">{text}</div>

              <div className="mx-auto px-8 md:w-1/2">
                <div className="aspect-card h-full w-full rounded-3xl bg-neutral-200">
                  <div className="relative h-full w-full object-cover">
                    {image &&
                      (isFromCDN(image) ? (
                        <Image
                          src={image}
                          priority
                          fill
                          sizes="(min-width: 768px) 50vw, 100vw"
                          alt=""
                          className="rounded-3xl object-cover"
                        />
                      ) : (
                        <img
                          src={image}
                          sizes="(min-width: 768px) 50vw, 100vw"
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
            {errorLoading ? (
              <div className="space-y-2 text-center text-lg">
                <div className="text-red-900">Error loading space. {errorLoading}</div>
                <button
                  onClick={() => window.location.reload()}
                  className="rounded-lg border border-neutral-500 px-4 py-1 hover:bg-neutral-100 active:bg-neutral-200"
                >
                  Try again
                </button>
              </div>
            ) : (
              <LoadingBar progress={loadingProgress} text={loadingText} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
