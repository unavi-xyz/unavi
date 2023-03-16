import { RefObject } from "react";

interface Props extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  textAreaRef?: RefObject<HTMLTextAreaElement>;
}

export default function TextArea({ name, textAreaRef, ...rest }: Props) {
  return (
    <label>
      {name && <div className="pb-1 text-lg font-bold">{name}</div>}

      <textarea
        ref={textAreaRef}
        name={name}
        className="h-full max-h-64 w-full rounded-md border border-neutral-200 px-3 py-2 text-neutral-900 placeholder:text-neutral-400"
        {...rest}
      />
    </label>
  );
}
