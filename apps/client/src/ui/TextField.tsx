import { RefObject, useId } from "react";
import { MdOutlineHelpOutline } from "react-icons/md";

import Tooltip from "./Tooltip";

interface Props extends React.InputHTMLAttributes<HTMLInputElement> {
  title?: string;
  frontAdornment?: string;
  help?: string;
  outline?: boolean;
  inputRef?: RefObject<HTMLInputElement>;
}

export default function TextField({
  title,
  frontAdornment,
  help,
  inputRef,
  outline = false,
  ...rest
}: Props) {
  const id = useId();

  const outlineClass = outline ? "border border-neutral-200" : "";

  return (
    <div className="flex flex-col space-y-1">
      <div className="flex items-center space-x-1">
        <label htmlFor={id} className="pointer-events-none block text-lg font-bold">
          {title}
        </label>

        {help && (
          <div>
            <Tooltip text={help} placement="right">
              <MdOutlineHelpOutline />
            </Tooltip>
          </div>
        )}
      </div>

      <div className={`flex items-center rounded bg-white ${outlineClass}`}>
        {frontAdornment && (
          <span className="pl-3 font-bold text-neutral-500">{frontAdornment}</span>
        )}
        <input
          ref={inputRef}
          id={id}
          type="text"
          className="h-full w-full rounded px-3 py-2 outline-none"
          {...rest}
        />
      </div>
    </div>
  );
}
