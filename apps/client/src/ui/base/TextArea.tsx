import { ChangeEvent, RefObject, useId } from "react";

interface Props {
  title?: string;
  frontAdornment?: string;
  textAreaRef?: RefObject<HTMLTextAreaElement>;
  onChange?: (e: ChangeEvent<HTMLTextAreaElement>) => void;
  [key: string]: any;
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
    <div className="w-full flex flex-col space-y-1">
      <label
        htmlFor={id}
        className="block text-lg font-bold pointer-events-none"
      >
        {title}
      </label>

      <div className="flex items-center border rounded-lg">
        {frontAdornment && (
          <span className="pl-2 text-primary font-bold">{frontAdornment}</span>
        )}
        <textarea
          ref={textAreaRef}
          id={id}
          className="outline-none w-full h-full px-3 py-2 rounded-md bg-surface text-onSurface"
          onChange={onChange}
          {...rest}
        />
      </div>
    </div>
  );
}
