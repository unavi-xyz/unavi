import { RefObject, useId } from "react";

interface Props extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  textAreaRef?: RefObject<HTMLTextAreaElement>;
}

export default function TextArea({ name, textAreaRef, ...rest }: Props) {
  const id = useId();

  return (
    <div className="flex w-full flex-col space-y-1">
      {name && (
        <label htmlFor={id} className="block text-lg font-bold">
          {name}
        </label>
      )}

      <textarea
        ref={textAreaRef}
        id={id}
        name={name}
        className="h-full max-h-64 w-full rounded-md border border-neutral-200 px-3 py-2 text-neutral-900 placeholder:text-neutral-400"
        {...rest}
      />
    </div>
  );
}
