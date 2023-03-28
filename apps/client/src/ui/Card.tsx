import Image from "next/image";

import { isFromCDN } from "../utils/isFromCDN";

interface Props {
  image?: string | null;
  text?: string | null;
  sizes?: string;
  loading?: boolean;
  loadingAnimation?: boolean;
  children?: React.ReactNode;
}

export default function Card({
  image,
  text,
  sizes = "(min-width: 1320px) 25vw, (min-width: 1024px) 33vw, (min-width: 768px) 50vw, 100vw",
  loading = false,
  loadingAnimation = true,
  children,
}: Props) {
  return (
    <div>
      <div
        className={`relative flex aspect-square h-full w-full flex-col overflow-hidden rounded-[2rem] bg-neutral-200 shadow-lg transition duration-100 ease-out ${
          loading ? "" : "hover:scale-105 hover:opacity-90"
        } ${loading && loadingAnimation ? "animate-pulse" : ""}`}
      >
        {loading ? null : image ? (
          isFromCDN(image) ? (
            <Image
              src={image}
              priority
              fill
              sizes={sizes}
              draggable={false}
              alt=""
              className="rounded-3xl object-cover"
            />
          ) : (
            <img
              src={image}
              draggable={false}
              alt=""
              className="h-full w-full rounded-3xl object-cover"
              crossOrigin="anonymous"
            />
          )
        ) : null}

        {children}
      </div>

      <div className="pt-2.5 pb-1">
        {loading ? null : <div className="text-xl font-bold text-neutral-900">{text}</div>}
      </div>
    </div>
  );
}
