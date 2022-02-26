import Pressable from "../../Pressable";

interface Props {
  icon?: any;
  image?: string;
  selected?: boolean;
  dark?: boolean;
  tooltip?: string;
}

export default function SidebarButton({
  icon,
  image,
  selected,
  dark,
  tooltip,
}: Props) {
  const colors = dark
    ? "bg-neutral-800 hover:bg-neutral-700 text-white"
    : "bg-white hover:bg-neutral-300 text-neutral-800";

  const round = selected ? "rounded-xl" : "rounded-3xl";

  const css = `group object-cover relative flex items-center justify-center h-12 w-12 my-2 mx-auto
              shadow-lg hover:cursor-pointer hover:rounded-xl transition-all
              ease-linear duration-150 text-xl ${colors} ${round}`;

  return (
    <Pressable>
      <button className={css}>
        {image ? <img src={image} alt="" className={css} /> : icon}

        {tooltip && (
          <span
            className="absolute w-auto p-2 m-2 min-w-max left-16 rounded-md shadow
                   text-white bg-neutral-900 text-xs font-bold transition-all
                     duration-150 scale-0 origin-left group-hover:scale-100"
          >
            {tooltip}
          </span>
        )}
      </button>
    </Pressable>
  );
}
