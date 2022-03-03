import { Dispatch, ReactChild, SetStateAction } from "react";
import ReactDOM from "react-dom";

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
  children: ReactChild | ReactChild[];
}
export default function Dialog({
  open,
  setOpen,
  children,
}: Props): JSX.Element {
  if (!open) return null;

  return ReactDOM.createPortal(
    <div
      className="fixed top-0 left-0 z-10 bg-black bg-opacity-50 w-screen
                 h-screen flex flex-col justify-center"
      onClick={() => setOpen(false)}
    >
      <dialog
        open
        onClick={(e) => e.stopPropagation()}
        className="fade mx-auto w-full max-w-xl flex flex-col space-y-4 p-12 rounded-lg"
      >
        {children}
      </dialog>
    </div>,
    document.body
  );
}
