"use client";

import * as Tabs from "@radix-ui/react-tabs";
import { ForwardRefExoticComponent, RefAttributes, useState } from "react";

interface Props {
  titles: string[];
  children: React.ReactNode;
}

export default function ButtonTabs({ titles, children }: Props) {
  const [value, setValue] = useState(titles[0]);

  return (
    <Tabs.Root
      defaultValue={titles[0]}
      value={value}
      onValueChange={setValue}
      className="space-y-4"
    >
      {titles.length > 1 && (
        <Tabs.List className="flex space-x-4">
          {titles.map((title) => (
            <Tabs.Trigger
              key={title}
              value={title}
              className={`flex-1 rounded-xl py-1 text-lg font-bold transition active:opacity-80 ${
                value === title
                  ? "bg-neutral-200 hover:bg-neutral-300"
                  : " hover:bg-neutral-200"
              }`}
            >
              {title}
            </Tabs.Trigger>
          ))}
        </Tabs.List>
      )}

      {children}
    </Tabs.Root>
  );
}

export const TabContent: ForwardRefExoticComponent<
  Tabs.TabsContentProps & RefAttributes<HTMLDivElement>
> = Tabs.Content;
