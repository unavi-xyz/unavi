import Image from "next/future/image";
import { useEffect, useRef, useState } from "react";

interface Props {
  image?: string | null;
  text?: string | null;
  subtext?: string | null;
  sizes?: string;
  aspect?: "card" | "vertical";
  animateEnter?: boolean;
}

export default function Card({
  image,
  text,
  subtext,
  sizes,
  aspect = "card",
  animateEnter = false,
}: Props) {
  const cardRef = useRef<HTMLDivElement>(null);
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
      ref={cardRef}
      className={`group p-2 w-full h-full overflow-hidden rounded-2xl hover:cursor-pointer
                  flex flex-col hover:ring-2 hover:ring-black ${opacityCss}`}
    >
      <div
        className={`h-full overflow-hidden rounded-xl ${aspectCss} bg-primaryContainer transition duration-300 ${opacityCss}`}
      >
        <div className="relative w-full h-full">
          {image && (
            <Image
              src={image}
              priority
              fill
              sizes={sizes}
              draggable={false}
              alt="card image"
              className="group-hover:scale-110 transition duration-500 ease-in-out rounded-xl"
            />
          )}
        </div>
      </div>

      <div className="space-y-2 py-1">
        {text && <div className="px-1 text-xl overflow-hidden">{text}</div>}
        {subtext && (
          <div className="px-1 text-lg overflow-hidden">{subtext}</div>
        )}
      </div>
    </div>
  );
}
