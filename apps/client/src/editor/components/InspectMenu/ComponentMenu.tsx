import { MdClose } from "react-icons/md";

interface Props {
  title?: string;
  removeable?: boolean;
  onRemove?: () => void;
  children?: React.ReactNode;
}

export default function ComponentMenu({ title, removeable = true, onRemove, children }: Props) {
  const outlineClass = removeable ? "hover:ring-1" : "";

  return (
    <div className={`group space-y-4 rounded-2xl p-4 ring-neutral-300 transition ${outlineClass}`}>
      <div className="-mt-1 flex justify-between text-xl font-bold">
        {title && <div>{title}</div>}

        {removeable && (
          <button
            onClick={onRemove}
            className="text-neutral-500 opacity-0 transition hover:text-black focus:opacity-100 group-hover:opacity-100"
          >
            <MdClose />
          </button>
        )}
      </div>

      {children && <div className="space-y-4">{children}</div>}
    </div>
  );
}
