import Link from "next/link";
import { BiMove } from "react-icons/bi";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import { FaPlay } from "react-icons/fa";
import { HiCubeTransparent } from "react-icons/hi";
import { MdArrowBackIosNew, MdSync } from "react-icons/md";

export default function Loading() {
  return (
    <div className="flex h-12 items-center justify-between border-b px-4 py-1">
      <div className="flex w-full items-center space-x-4">
        <div className="flex w-full items-center space-x-2 text-lg">
          <Link
            href="/"
            className="cursor-pointer p-1 text-neutral-500 transition hover:text-inherit"
          >
            <MdArrowBackIosNew />
          </Link>

          <div className="h-8 w-40 animate-pulse rounded-md bg-neutral-200" />
        </div>

        <div className="flex h-full w-full justify-center space-x-2">
          <SkeletonIconButton>
            <BiMove />
          </SkeletonIconButton>

          <SkeletonIconButton>
            <MdSync />
          </SkeletonIconButton>

          <SkeletonIconButton>
            <CgArrowsExpandUpRight />
          </SkeletonIconButton>
        </div>

        <div className="flex h-full w-full items-center justify-end space-x-2">
          <SkeletonIconButton>
            <FaPlay className="text-sm" />
          </SkeletonIconButton>

          <SkeletonIconButton>
            <HiCubeTransparent />
          </SkeletonIconButton>

          <div className="cursor-default rounded-full bg-neutral-900 px-6 py-1.5 font-bold text-white outline-neutral-400">
            Sign in
          </div>
        </div>
      </div>
    </div>
  );
}

function SkeletonIconButton({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex aspect-square h-full cursor-default items-center justify-center text-2xl">
      {children}
    </div>
  );
}
