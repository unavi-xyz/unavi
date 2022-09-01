import Image from "next/future/image";

interface Props {
  image?: string;
  text?: string;
  subtext?: string;
  sizes?: string;
  aspect?: "card" | "vertical";
}

export default function Card({
  image,
  text,
  subtext,
  sizes,
  aspect = "card",
}: Props) {
  const aspectCss = aspect === "card" ? "aspect-card" : "aspect-vertical";

  return (
    <div
      className={`group p-3 w-full h-full overflow-hidden rounded-2xl hover:cursor-pointer
                  flex flex-col hover:ring-2 hover:ring-outline`}
    >
      <div
        className={`h-full overflow-hidden rounded-xl ${aspectCss} bg-primaryContainer`}
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

      <div className="space-y-2 pt-2">
        {text && <div className="px-1 text-xl overflow-hidden">{text}</div>}
        {subtext && (
          <div className="px-1 text-lg overflow-hidden">{subtext}</div>
        )}
      </div>
    </div>
  );
}
