import React from "react";

interface Props {
  children: React.ReactNode;
}

/**
 * A responsive grid for {@link Card} components.
 */
export default function CardGrid({ children }: Props) {
  return (
    <div className="grid w-full grid-cols-1 gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
      {children}
    </div>
  );
}
