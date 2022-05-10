import { useState, useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { atom, useAtomValue, useSetAtom } from "jotai";

const closeDialogAtom = atom(false);

interface Props {
  open: boolean;
  onClose: () => void;
  children: React.ReactNode;
}

export default function Dialog({ open, onClose, children }: Props) {
  const dialogRef = useRef<HTMLDivElement>(null);
  const backdropRef = useRef<HTMLDivElement>(null);

  const closeDialog = useAtomValue(closeDialogAtom);

  const [visible, setVisible] = useState(false);

  useEffect(() => {
    //add a delay before unmounting the menu so we can show animations
    const timeout = setTimeout(() => setVisible(open), 200);

    //dont add the delay on open
    if (open) {
      setVisible(true);
      clearTimeout(timeout);
    }

    //open / close animation
    if (open) {
      setTimeout(() => {
        dialogRef.current?.classList.remove("scale-75");
        dialogRef.current?.classList.remove("opacity-0");
        backdropRef.current?.classList.remove("opacity-0");
      }, 10);
    } else {
      dialogRef.current?.classList.add("opacity-0");
      dialogRef.current?.classList.add("scale-75");
      backdropRef.current?.classList.add("opacity-0");
    }

    return () => clearTimeout(timeout);
  }, [open]);

  useEffect(() => {
    if (closeDialog) onClose();
  }, [closeDialog, onClose]);

  if (!visible) return null;

  return createPortal(
    <div
      ref={backdropRef}
      onMouseDown={onClose}
      className="fixed top-0 left-0 bg-black bg-opacity-50 w-screen
                 h-screen flex flex-col justify-center z-50 opacity-0
                 transition-opacity duration-200 ease-in-out"
    >
      <dialog
        ref={dialogRef}
        open
        onMouseDown={(e) => e.stopPropagation()}
        className="rounded-2xl p-12 fade mx-auto w-full max-w-xl flex flex-col space-y-4
                   transition-all duration-200 ease-in-out scale-75 opacity-0"
      >
        {children}
      </dialog>
    </div>,
    document.body
  );
}

export function useCloseDialog() {
  const setOpenDialog = useSetAtom(closeDialogAtom);

  function close() {
    setOpenDialog(false);
  }

  return close;
}
