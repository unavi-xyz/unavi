"use client";

import { useRouter } from "next/navigation";
import { MdArrowBackIos } from "react-icons/md";

import { useSave } from "../../hooks/useSave";

interface Props {
  projectId: string;
}

export default function BackButton({ projectId }: Props) {
  const { save } = useSave(projectId);
  const router = useRouter();

  async function handleBack() {
    await save();
    router.push("/");
  }

  return (
    <button
      onClick={handleBack}
      className="flex h-8 w-fit items-center text-neutral-600 hover:text-black"
    >
      <MdArrowBackIos />
    </button>
  );
}
