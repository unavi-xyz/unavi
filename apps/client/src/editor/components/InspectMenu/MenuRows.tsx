interface Props {
  titles: string[];
  children: React.ReactNode;
}

export default function MenuRows({ titles, children }: Props) {
  return (
    <div className="flex h-full items-stretch space-x-3">
      <div className="flex w-1/3 min-w-fit flex-col justify-between">
        {titles.map((title) => (
          <div key={title}>{title}</div>
        ))}
      </div>

      <div className="flex w-full flex-col justify-between space-y-1">{children}</div>
    </div>
  );
}
