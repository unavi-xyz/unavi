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
    <section className="relative w-full overflow-hidden rounded-3xl bg-gradient-to-tl from-orange-300 to-yellow-200 p-8">
      <div className="absolute -bottom-4 -left-12 -rotate-12 select-none text-8xl opacity-50 md:text-9xl">
        🏗️
      </div>

      <div className="flex h-full flex-col items-center justify-center space-y-2">
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
