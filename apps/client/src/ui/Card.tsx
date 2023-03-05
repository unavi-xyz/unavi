import Image from "next/image";

import { isFromCDN } from "../utils/isFromCDN";

interface Props {
  image?: string | null;
  text?: string | null;
  sizes?: string;
  aspect?: "card" | "vertical";
  animateEnter?: boolean;
  loading?: boolean;
  loadingAnimation?: boolean;
  children?: React.ReactNode;
}

export default function Card({
  image,
  text,
  sizes,
  aspect = "card",
  animateEnter = false,
  loading = false,
  loadingAnimation = true,
  children,
}: Props) {
  const aspectCss = aspect === "card" ? "aspect-card" : "aspect-vertical";
  const animateCss = animateEnter ? "animate-floatIn" : "";

  return (
    <div className={`h-full w-full transition ${loading ? "" : "hover:scale-105"}`}>
      <div
        className={`relative flex h-full w-full flex-col overflow-hidden rounded-xl bg-neutral-200 ${
          loading && loadingAnimation ? "animate-pulse" : ""
        } ${animateCss} ${aspectCss}`}
      >
        {image &&
          (isFromCDN(image) ? (
            <Image
              src={image}
              priority
              fill
              sizes={sizes}
              draggable={false}
              alt=""
              className="rounded-xl object-cover"
            />
          ) : (
            <img
              src={image}
              draggable={false}
              alt=""
              className="h-full w-full rounded-xl object-cover"
              crossOrigin="anonymous"
            />
          ))}

        <div className="absolute flex h-full w-full items-end tracking-wide text-white">
          {text && (
            <div
              className="w-full overflow-hidden px-3 pb-2 text-xl font-black drop-shadow-dark"
              style={{ textShadow: "0 0 6px rgba(0, 0, 0, 0.6)" }}
            >
              {text}
            </div>
          )}
        </div>

        {children}
      </div>
    </div>
  );
}
