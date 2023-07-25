import {
  forwardRef,
  InputHTMLAttributes,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

const DISABLE_ARROWS =
  "[appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none";

interface Props extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  value: number;
  sensitivity?: number;
  updateInterval?: number;
  precision?: number;
}

const NumberInput = forwardRef<HTMLInputElement, Props>(
  (
    {
      label,
      className,
      value,
      onChange,
      disabled,
      sensitivity = 50,
      updateInterval = 10,
      precision = 100,
      ...rest
    },
    ref
  ) => {
    const displayedRef = useRef(value);
    const [displayed, setDisplayed] = useState(value);
    const [mouseDown, setMouseDown] = useState(false);

    useEffect(() => {
      displayedRef.current = value;
      setDisplayed(value);
    }, [value]);

    const onPointerDown = useCallback(
      (event: React.PointerEvent<HTMLInputElement>) => {
        event.preventDefault();

        if (disabled) return;

        setMouseDown(true);

        const target = event.currentTarget;

        target.setPointerCapture(event.pointerId);
        target.requestPointerLock();

        const callOnChange = () => {
          if (onChange) {
            onChange({ target: { value: displayedRef.current } } as any);
          }
        };

        const interval = setInterval(callOnChange, updateInterval);

        const startDisplayed = displayedRef.current;

        let moveCount = 0;
        let totalDiff = 0;

        const onPointerMove = (e: PointerEvent) => {
          moveCount++;

          totalDiff += e.movementX;
          const percentDiff = totalDiff / window.innerWidth;

          const newDisplayed = round(
            startDisplayed + percentDiff * sensitivity,
            precision
          );

          displayedRef.current = newDisplayed;
          setDisplayed(newDisplayed);
        };

        const onPointerUp = (e: PointerEvent) => {
          e.stopPropagation();

          // If didn't move much, focus the input
          if (moveCount < 5) {
            target.focus();
          }

          target.releasePointerCapture(event.pointerId);
          document.exitPointerLock();

          setMouseDown(false);

          callOnChange();
          clearInterval(interval);

          target.removeEventListener("pointermove", onPointerMove);
          target.removeEventListener("pointerup", onPointerUp);
        };

        target.addEventListener("pointermove", onPointerMove);
        target.addEventListener("pointerup", onPointerUp);
      },
      [disabled, updateInterval, onChange, sensitivity, precision]
    );

    return (
      <label className="flex items-baseline text-sm">
        <span className="font-semibold text-neutral-500">{label}</span>

        <input
          ref={ref}
          type="number"
          value={displayed}
          disabled={disabled}
          onPointerDown={onPointerDown}
          onChange={onChange}
          className={`w-full cursor-ew-resize rounded bg-inherit px-1.5 py-1 text-neutral-200 placeholder:text-neutral-400 ${
            disabled ? "cursor-not-allowed text-opacity-50" : ""
          } ${mouseDown ? "cursor-none" : ""} ${DISABLE_ARROWS} ${className}`}
          {...rest}
        />
      </label>
    );
  }
);

NumberInput.displayName = "NumberInput";

export default NumberInput;

function round(num: number, precision: number) {
  return Math.round(num * precision) / precision;
}
