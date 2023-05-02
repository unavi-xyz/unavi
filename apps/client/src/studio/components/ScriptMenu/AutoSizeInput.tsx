import { CSSProperties, HTMLProps, useCallback, useEffect, useRef, useState } from "react";

interface Props extends HTMLProps<HTMLInputElement> {
  minWidth?: number;
}

const baseStyles: CSSProperties = {
  height: 0,
  left: 0,
  position: "absolute",
  top: 0,
  visibility: "hidden",
  whiteSpace: "pre",
  width: "auto",
};

export default function AutoSizeInput({ minWidth = 40, ...props }: Props) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const measureRef = useRef<HTMLSpanElement | null>(null);
  const [styles, setStyles] = useState<CSSProperties>({});

  // grab the font size of the input on ref mount
  const setRef = useCallback((input: HTMLInputElement | null) => {
    if (input) {
      const styles = window.getComputedStyle(input);
      setStyles({
        fontSize: styles.getPropertyValue("font-size"),
        paddingLeft: styles.getPropertyValue("padding-left"),
        paddingRight: styles.getPropertyValue("padding-right"),
      });
    }
    inputRef.current = input;
  }, []);

  // measure the text on change and update input
  useEffect(() => {
    if (measureRef.current === null) return;
    if (inputRef.current === null) return;

    const width = measureRef.current.clientWidth;
    const extraWidth = props.type === "number" ? 20 : 0;

    inputRef.current.style.width = Math.max(minWidth, width) + extraWidth + "px";
  }, [props.value, props.type, minWidth, styles]);

  return (
    <>
      <input ref={setRef} {...props} />
      <span ref={measureRef} style={{ ...baseStyles, ...styles }}>
        {props.value}
      </span>
    </>
  );
}
