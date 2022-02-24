export default function RichButton({
  image = <></>,
  icon = <></>,
  title = "",
  description = "",
  ...rest
}) {
  return (
    <div
      {...rest}
      className="flex space-x-2 p-1 shadow-md bg-slate-100 hover:bg-slate-200
                 hover:cursor-pointer rounded-lg h-24"
    >
      {image}

      {icon && (
        <div className="flex items-center text-4xl px-4 text-neutral-500">
          {icon}
        </div>
      )}

      <div className="flex flex-col justify-center space-y-1">
        <h3 className="text-xl">{title}</h3>
        <p className="text-neutral-500">{description}</p>
      </div>
    </div>
  );
}
