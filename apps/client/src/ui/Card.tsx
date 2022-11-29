import Image from "next/image";
import { useEffect, useState } from "react";
import { MdPeople } from "react-icons/md";

import { isFromCDN } from "../utils/isFromCDN";

interface Props {
  image?: string | null;
  text?: string | null;
  subtext?: string | null;
  sizes?: string;
  aspect?: "card" | "vertical";
  animateEnter?: boolean;
  playerCount?: number;
}

export default function Card({
  image,
  text,
  subtext,
  sizes,
  aspect = "card",
  animateEnter = false,
  playerCount,
}: Props) {
  const [visible, setVisible] = useState(!animateEnter);

  useEffect(() => {
    if (!animateEnter) return;
    const timeout = setTimeout(() => setVisible(true), 100);
    return () => clearTimeout(timeout);
  }, [animateEnter]);

  const aspectCss = aspect === "card" ? "aspect-card" : "aspect-vertical";
  const opacityCss = visible ? "opacity-100" : "opacity-0 translate-y-2";

  return (
    <div
      className={`relative flex h-full w-full flex-col overflow-hidden rounded-2xl bg-primaryContainer transition hover:scale-105 hover:cursor-pointer ${opacityCss} ${aspectCss}`}
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
            className="rounded-2xl object-cover"
          />
        ) : (
          <img
            src={image}
            draggable={false}
            alt=""
            className="h-full w-full rounded-2xl object-cover"
            crossOrigin="anonymous"
          />
        ))}

      <div className="absolute flex h-full w-full items-start p-2 tracking-wide">
        {playerCount !== undefined && playerCount > 0 && (
          <div className="flex items-center space-x-1.5 rounded-full bg-black/50 px-3 py-0.5 text-white  backdrop-blur-lg">
            <MdPeople className="text-lg" />
            <div className="font-bold">{playerCount}</div>
          </div>
        )}
      </div>

      <div className="absolute flex h-full w-full items-end px-3 pb-2 tracking-wide text-white">
        {text && (
          <div
            className="w-full overflow-hidden text-xl font-black drop-shadow-dark"
            style={{
              textShadow: "0 0 6px rgba(0, 0, 0, 0.4)",
            }}
          >
            {text}
          </div>
        )}

        {subtext && (
          <div
            className="overflow-hidden text-lg drop-shadow-dark"
            style={{
              textShadow: "0 0 6px rgba(0, 0, 0, 0.4)",
            }}
          >
            {subtext}
          </div>
        )}
      </div>
    </div>
  );
}
