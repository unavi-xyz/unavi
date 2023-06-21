import Image from "next/image";
import Link from "next/link";

import { isFromCDN } from "../utils/isFromCDN";

interface Props {
  href?: string;
  image?: string | null;
  text?: string | null;
  sizes?: string;
  loading?: boolean;
  loadingAnimation?: boolean;
  children?: React.ReactNode;
}

export default function Card({
  href,
  image,
  text,
  sizes,
  loading,
  loadingAnimation,
  children,
}: Props) {
  return (
    <div>
      {href ? (
        <Link href={href} className="block">
          <CardImage
            image={image}
            sizes={sizes}
            loading={loading}
            loadingAnimation={loadingAnimation}
          >
            {children}
          </CardImage>
        </Link>
      ) : (
        <CardImage
          image={image}
          sizes={sizes}
          loading={loading}
          loadingAnimation={loadingAnimation}
        >
          {children}
        </CardImage>
      )}

      {loading ? null : <CardText text={text} />}
    </div>
  );
}

export function CardText({ text }: { text?: string | null }) {
  return (
    <div className="pb-1 pt-2.5 text-xl font-bold text-neutral-900">{text}</div>
  );
}

interface CardImageProps {
  group?: boolean;
  image?: string | null;
  sizes?: string;
  loading?: boolean;
  loadingAnimation?: boolean;
  children: React.ReactNode;
}

export function CardImage({
  group = false,
  image,
  sizes = "(min-width: 1320px) 25vw, (min-width: 1024px) 33vw, (min-width: 768px) 50vw, 100vw",
  loading = false,
  loadingAnimation = true,
  children,
}: CardImageProps) {
  return (
    <div
      className={`aspect-card relative flex h-full w-full flex-col overflow-hidden rounded-3xl bg-neutral-200 transition duration-100 ease-out ${
        loading
          ? ""
          : group
          ? "group-hover:scale-105 group-hover:shadow-lg"
          : "hover:scale-105 hover:shadow-lg"
      } ${loading && loadingAnimation ? "animate-pulse" : ""}`}
    >
      {loading ? null : image ? (
        isFromCDN(image) ? (
          <Image
            src={image}
            priority
            fill
            sizes={sizes}
            draggable={false}
            alt=""
            className="rounded-3xl object-cover"
          />
        ) : (
          <img
            src={image}
            draggable={false}
            alt=""
            className="h-full w-full rounded-3xl object-cover"
            crossOrigin="anonymous"
          />
        )
      ) : null}

      {children}
    </div>
  );
}
