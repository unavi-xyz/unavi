interface Props {
  icon?: any;
  image?: string;
  selected?: boolean;
  dark?: boolean;
}

export default function SidebarButton({ icon, image, selected, dark }: Props) {
  const colors = dark
    ? "bg-neutral-800 hover:bg-neutral-700 text-white"
    : "bg-white hover:bg-neutral-300 text-neutral-800";

  const round = selected ? "rounded-xl" : "rounded-3xl";

  const css = `object-cover relative flex items-center justify-center h-12 w-12 my-2 mx-auto
              shadow-lg hover:cursor-pointer hover:rounded-xl transition-all
              ease-linear duration-150 text-xl ${colors} ${round}`;

  if (image) return <img src={image} alt="" className={css} />;

  return <div className={css}>{icon}</div>;
}
