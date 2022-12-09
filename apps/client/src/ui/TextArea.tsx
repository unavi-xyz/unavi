import { RefObject, useId } from "react";

interface Props extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  title?: string;
  frontAdornment?: string;
  outline?: boolean;
  textAreaRef?: RefObject<HTMLTextAreaElement>;
}

export default function TextArea({
  title,
  textAreaRef,
  outline,
  frontAdornment,
  ...rest
}: Props) {
  const id = useId();

  const outlineClass = outline ? "border border-neutral-200" : "";

  return (
    <div className="flex w-full flex-col space-y-1">
      <label
        htmlFor={id}
        className="pointer-events-none block text-lg font-bold"
      >
        {title}
      </label>

      <div className="flex items-center rounded">
        {frontAdornment && (
          <span className="pl-2 font-bold text-sky-300">{frontAdornment}</span>
        )}
        <textarea
          ref={textAreaRef}
          id={id}
          className={`h-full max-h-64 w-full rounded bg-white px-3 py-2 outline-none ${outlineClass}`}
          {...rest}
        />
      </div>
    </div>
  );
}
