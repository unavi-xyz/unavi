import { useEffect, useRef, useState } from "react";

type Props = React.InputHTMLAttributes<HTMLInputElement>;

export default function AutoGrowInput(props: Props) {
  const span = useRef<HTMLSpanElement>(null);

  const [content, setContent] = useState(props.value);
  const [width, setWidth] = useState(0);

  useEffect(() => {
    if (!span.current) return;
    setWidth(span.current.offsetWidth);
    setContent(props.value);
  }, [props.value]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (props.onChange) props.onChange(e);
  };

  return (
    <>
      <span ref={span} className="absolute -z-50 whitespace-pre py-0.5 pl-3 pr-3.5 opacity-0">
        {props.value}
      </span>
      <input
        {...props}
        value={content}
        style={{ width }}
        onChange={handleChange}
        className="max-w-full rounded-lg py-0.5 pl-3 pr-3.5 transition hover:bg-neutral-100 hover:shadow-inner"
      />
    </>
  );
}
