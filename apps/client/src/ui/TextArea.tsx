import { RefObject, useId } from "react";

interface Props extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  frontAdornment?: string;
  outline?: boolean;
  textAreaRef?: RefObject<HTMLTextAreaElement>;
}

export default function TextArea({ name, textAreaRef, outline, frontAdornment, ...rest }: Props) {
  const id = useId();

  const outlineClass = outline ? "border border-neutral-200" : "";

  return (
    <div className="flex w-full flex-col space-y-1">
      <label htmlFor={id} className="pointer-events-none block text-lg font-bold">
        {name}
      </label>

      <div className="flex items-center rounded-md">
        {frontAdornment && <span className="pl-2 font-bold text-sky-300">{frontAdornment}</span>}
        <textarea
          ref={textAreaRef}
          id={id}
          name={name}
          className={`h-full max-h-64 w-full rounded-md px-3 py-2 outline-none ${outlineClass}`}
          {...rest}
        />
      </div>
    </div>
  );
}
