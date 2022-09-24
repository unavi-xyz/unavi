import { RefObject, useId } from "react";

interface Props extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  title?: string;
  frontAdornment?: string;
  textAreaRef?: RefObject<HTMLTextAreaElement>;
}

export default function TextArea({
  title,
  textAreaRef,
  onChange,
  frontAdornment,
  ...rest
}: Props) {
  const id = useId();

  return (
    <div className="flex w-full flex-col space-y-1">
      <label
        htmlFor={id}
        className="pointer-events-none block text-lg font-bold"
      >
        {title}
      </label>

      <div className="flex items-center rounded-lg">
        {frontAdornment && (
          <span className="pl-2 font-bold text-primary">{frontAdornment}</span>
        )}
        <textarea
          ref={textAreaRef}
          id={id}
          className="h-full w-full rounded-md bg-surface px-3 py-2 text-onSurface outline-none"
          onChange={onChange}
          {...rest}
        />
      </div>
    </div>
  );
}
