import { useEffect, useRef, useState } from "react";

interface Props {
  open?: boolean;
  onClose?: () => void;
  placement?: "left" | "right";
  children: React.ReactNode;
}

export default function DropdownMenu({
  open = false,
  onClose,
  placement = "left",
  children,
}: Props) {
  const menuRef = useRef<HTMLDivElement>(null);

  const [visible, setVisible] = useState(false);

  useEffect(() => {
    // Add a delay before unmounting the menu so we can show animations
    const timeout = setTimeout(() => setVisible(open), 150);

    // Don't add the delay on open
    if (open) {
      setVisible(true);
      clearTimeout(timeout);
    }

    // Open close animations
    if (open) {
      menuRef.current?.classList.remove("scale-75");
      menuRef.current?.classList.remove("opacity-0");
    } else {
      menuRef.current?.classList.add("opacity-0");
      menuRef.current?.classList.add("scale-75");
    }

    return () => clearTimeout(timeout);
  }, [open]);

  useEffect(() => {
    //if the user clicks outside of the dropdown, close it
    function onPointerUp() {
      if (open && onClose) onClose();
    }

    document.addEventListener("pointerup", onPointerUp);
    return () => {
      document.removeEventListener("pointerup", onPointerUp);
    };
  }, [onClose, open]);

  const placementClass = placement === "left" ? "left-0" : "right-0";

  return (
    <div className="relative">
      <div
        ref={menuRef}
        className={`absolute z-10 w-full min-w-max rounded-xl bg-surface
                    text-onSurface shadow-lg transition ease-in-out ${placementClass}
                    scale-75 opacity-0`}
      >
        {visible && children}
      </div>
    </div>
  );
}
