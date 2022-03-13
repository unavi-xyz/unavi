interface Props {
  text?: string;
  image?: string;
}

export default function Card({ text, image }: Props) {
  return (
    <div
      className="relative w-full h-full rounded-lg hover:cursor-pointer
                 flex flex-col"
    >
      <div className="h-full">
        {image && (
          <img
            src={image}
            alt={text}
            className="w-full h-full object-cover rounded-lg opacity-100"
          />
        )}
      </div>

      {text && (
        <div className="absolute bottom-0 px-3 py-2 text-lg rounded-b-lg w-full bg-neutral-100">
          {text}
        </div>
      )}
    </div>
  );
}
