"use client";

import { IoIosArrowDown } from "react-icons/io";

import { useStudio } from "../../Studio";

export default function TitleDisplay() {
  const { title } = useStudio();

  return (
    <div className="flex items-center space-x-2 rounded-lg px-2 py-1 transition hover:bg-neutral-200">
      <div className="text-lg">{title}</div>
      <IoIosArrowDown className="text-neutral-600" />
    </div>
  );
}
