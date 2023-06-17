"use client";

import { useAuth } from "@/src/client/AuthProvider";
import Button from "@/src/ui/Button";

import { useSignInStore } from "./signInStore";

export default function CreateCard() {
  const { status, loading } = useAuth();

  const disabled = status === "loading" || loading;

  function handleClick() {
    if (disabled) return;

    if (status === "unauthenticated") {
      useSignInStore.setState({ open: true });
      return;
    }
  }

  return (
    <section className="relative w-full overflow-hidden rounded-3xl bg-gradient-to-tl from-red-300 to-amber-200 p-6">
      <div className="absolute -bottom-4 -left-12 -rotate-12 select-none text-8xl opacity-40 md:text-9xl lg:left-[8%]">
        🏗️
      </div>

      <div className="flex flex-col items-center justify-center space-y-4">
        <h2 className="z-10 text-center text-3xl font-bold">
          Create your world. <br /> Share it with others.
        </h2>

        <Button
          onClick={handleClick}
          disabled={disabled}
          className="z-10 text-lg"
        >
          Start building
        </Button>
      </div>
    </section>
  );
}
