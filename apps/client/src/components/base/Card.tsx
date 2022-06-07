interface Props {
  image?: string;
  text?: string;
  subtext?: string;
  aspect?: "card" | "vertical";
}

export default function Card({ image, text, subtext, aspect = "card" }: Props) {
  const aspectCss = aspect === "card" ? "aspect-card" : "aspect-vertical";

  return (
    <div
      className={`group p-4 w-full h-full overflow-hidden rounded-3xl hover:cursor-pointer
                  flex flex-col hover:ring-2 hover:ring-outline ${aspectCss}`}
    >
      <div className="h-full overflow-hidden rounded-2xl bg-secondaryContainer">
        {image && (
          <img
            src={image}
            alt="card image"
            draggable={false}
            className="w-full h-full object-cover group-hover:scale-110
                       transition duration-500 ease-in-out"
          />
        )}
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
