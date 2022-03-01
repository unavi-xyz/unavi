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
    ? "bg-neutral-800 text-white"
    : "bg-white text-neutral-800";

  const css = `object-cover relative flex items-center justify-center h-12 w-12
              shadow-lg transition-all ease-linear duration-150 text-xl ${colors} rounded-full`;

  const bg = selected ? "bg-opacity-10" : "bg-opacity-0";

  return (
    <Pressable>
      <div
        className={`flex items-center space-x-4 bg-white ${bg} rounded-2xl
                    hover:bg-opacity-10 p-2 transition-all duration-150 hover:cursor-pointer`}
      >
        <button className={css}>
          {image ? <img src={image} alt="" className={css} /> : icon}
        </button>

        <div className="">{tooltip}</div>
      </div>
    </Pressable>
  );
}
