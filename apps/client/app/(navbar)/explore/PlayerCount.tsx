"use client";

import { MdPeople } from "react-icons/md";
import useSWR from "swr";

import { fetcher } from "@/src/play/utils/fetcher";

interface Props {
  id: number;
}

export default function PlayerCount({ id }: Props) {
  const { data: playerCount } = useSWR<number>(`/api/spaces/${id}/player-count`, fetcher);

  if (!playerCount) return null;

  return (
    <div className="absolute flex h-full w-full items-start justify-end p-4 tracking-wide">
      <div className="flex items-center space-x-2 rounded-full bg-neutral-800/60 px-3.5 py-0.5 text-white  backdrop-blur-lg">
        <MdPeople className="text-xl" />
        <div className="text-lg font-bold">{playerCount}</div>
      </div>
    </div>
  );
}
