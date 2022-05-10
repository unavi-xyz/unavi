interface Props {
  image?: string;
  text?: string;
  subtext?: string;
}

export default function Card({ image, text, subtext }: Props) {
  return (
    <div
      className="group p-2.5 aspect-card w-full h-full overflow-hidden rounded-3xl hover:cursor-pointer
                 flex flex-col hover:shadow-lg transition-all duration-300"
    >
      <div className="h-full overflow-hidden rounded-2xl bg-neutral-200 border">
        {image && (
          <img
            src={image}
            alt="card image"
            className="w-full h-full object-cover group-hover:scale-110
                       transition-all duration-500 ease-in-out"
          />
        )}
      </div>

      <div className="space-y-2 py-3">
        {text && <div className="px-1 text-xl overflow-hidden">{text}</div>}
        {subtext && (
          <div className="px-1 text-lg  overflow-hidden text-neutral-500">
            {subtext}
          </div>
        )}
      </div>
    </div>
  );
}
