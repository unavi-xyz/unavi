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
  onValueChange?: (value: number) => void;
}

const NumberInput = forwardRef<HTMLInputElement, Props>(
  (
    {
      label,
      className,
      value,
      disabled,
      min,
      sensitivity = 50,
      updateInterval = 15,
      precision = 100,
      onValueChange,
      onChange,
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

        const callValueChange = () => {
          if (onValueChange) {
            onValueChange(displayedRef.current);
          }
        };

        const interval = setInterval(callValueChange, updateInterval);

        const startDisplayed = displayedRef.current;

        let moveCount = 0;
        let totalDiff = 0;

        const onPointerMove = (e: PointerEvent) => {
          moveCount++;

          // Sometimes the movement is huge, so we need to clamp it
          const movement = Math.max(Math.min(e.movementX, 10), -10);
          totalDiff += movement;

          const percentDiff = totalDiff / window.innerWidth;

          let newDisplayed = round(
            startDisplayed + percentDiff * sensitivity,
            precision
          );

          if (min !== undefined && newDisplayed < Number(min)) {
            newDisplayed = Number(min);
          }

          displayedRef.current = newDisplayed;
          setDisplayed(newDisplayed);
        };

        const onPointerUp = (e: PointerEvent) => {
          e.stopPropagation();

          // If didn't move much, focus the input
          if (moveCount < 5) {
            target.focus();
            target.type = "text";
            target.setSelectionRange(0, target.value.length);
            target.type = "number";
          }

          target.releasePointerCapture(event.pointerId);
          document.exitPointerLock();

          setMouseDown(false);

          callValueChange();
          clearInterval(interval);

          target.removeEventListener("pointermove", onPointerMove);
          target.removeEventListener("pointerup", onPointerUp);
        };

        target.addEventListener("pointermove", onPointerMove);
        target.addEventListener("pointerup", onPointerUp);
      },
      [disabled, updateInterval, onValueChange, sensitivity, min, precision]
    );

    return (
      <label className="flex items-baseline text-sm">
        <span className="font-semibold text-neutral-500">{label}</span>

        <input
          ref={ref}
          type="number"
          min={min}
          value={displayed}
          disabled={disabled}
          onPointerDown={onPointerDown}
          onChange={(e) => {
            // If focused, call onValueChange
            // We don't want to call it if dragging, it will spam too much
            if (document.activeElement === e.currentTarget) {
              const value = e.currentTarget.value;
              onValueChange?.(Number(value));
              setDisplayed(Number(value));
              displayedRef.current = Number(value);
            }

            onChange?.(e);
          }}
          className={`w-full rounded bg-inherit px-1.5 py-1 text-neutral-200 placeholder:text-neutral-400 ${
            disabled ? "cursor-default text-opacity-50" : "cursor-ew-resize"
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
