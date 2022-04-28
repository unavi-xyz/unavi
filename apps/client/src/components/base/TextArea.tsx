import { ChangeEvent, RefObject } from "react";

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
  return (
    <div className="flex flex-col space-y-1">
      <label
        htmlFor={title}
        className="block text-lg font-bold pointer-events-none"
      >
        {title}
      </label>

      <div className="flex items-center border rounded-lg">
        {frontAdornment && (
          <span className="pl-2 text-neutral-500 font-bold">
            {frontAdornment}
          </span>
        )}
        <textarea
          ref={textAreaRef}
          id={title}
          className="outline-none w-full h-full px-3 py-2 rounded-lg"
          onChange={onChange}
          {...rest}
        />
      </div>
    </div>
  );
}
