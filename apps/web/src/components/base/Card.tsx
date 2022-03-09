interface Props {
  text?: string;
  image?: string;
}

export default function Card({ text, image }: Props) {
  return (
    <div
      className="relative w-full h-full bg-neutral-200 rounded-md hover:cursor-pointer
                 flex flex-col shadow"
    >
      <div className="h-full">
        {image && (
          <img
            src={image}
            alt={text}
            className="w-full h-full object-cover rounded-md opacity-100"
          />
        )}
      </div>

      {text && (
        <div
          className="absolute bottom-0 px-3 py-2 text-lg rounded-b-md w-full bg-white
                   bg-opacity-60 bg-blur-filter"
        >
          {text}
        </div>
      )}
    </div>
  );
}
