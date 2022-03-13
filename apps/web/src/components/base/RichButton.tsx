export function RichButton({
  image = <></>,
  icon = <></>,
  title = "",
  description = "",
  ...rest
}) {
  return (
    <div
      {...rest}
      className="flex w-full space-x-2 p-1 bg-neutral-100 hover:bg-neutral-200
                 hover:cursor-pointer rounded-lg h-24 transition-all duration-150"
    >
      {image}

      {icon && (
        <div className="flex items-center text-4xl px-4 text-neutral-500">
          {icon}
        </div>
      )}

      <div className="flex flex-col justify-center space-y-1">
        {title && <h3 className="text-lg">{title}</h3>}{" "}
        {description && (
          <p className="text-neutral-500 text-sm">{description}</p>
        )}{" "}
      </div>
    </div>
  );
}
