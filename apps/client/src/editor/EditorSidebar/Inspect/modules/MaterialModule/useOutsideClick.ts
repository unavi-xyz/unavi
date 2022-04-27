import { MutableRefObject, useEffect } from "react";

export function useOutsideClick(
  ref: MutableRefObject<any>,
  onClick: () => void
) {
  useEffect(() => {
    function handleClickOutside(event) {
      if (ref.current && !ref.current.contains(event.target)) {
        onClick();
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [ref, onClick]);
}
