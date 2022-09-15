import Image from "next/future/image";
import Link from "next/link";

import Button from "./base/Button";

interface Props {
  title: string;
  subtitle: string;
  body: string;
  buttonText: string;
  buttonLink: string;
  image: string;
  imageSide: "left" | "right";
}

export default function LandingInfoBlock({
  title,
  subtitle,
  body,
  buttonText,
  buttonLink,
  image,
  imageSide,
}: Props) {
  const directionClass =
    imageSide === "right" ? "flex-row-reverse" : "flex-row";

  const isExternalLink = buttonLink.startsWith("http");

  return (
    <div className={`flex h-screen snap-center items-center ${directionClass}`}>
      <div className="md:mt-8 md:h-1/2 md:w-1/2 md:p-8">
        <div className="relative h-full w-full">
          <Image
            src={image}
            fill
            sizes="293px"
            alt="Wired-chan"
            className="select-none object-contain"
          />
        </div>
      </div>

      <div className="w-full space-y-3">
        <div
          className="text-onPrimaryContainer bg-primaryContainer w-fit rounded-xl px-5 py-2
                     text-6xl font-black"
        >
          {title}
        </div>

        <div className="text-5xl">{subtitle}</div>

        <div className="text-outline ml-1 text-xl">{body}</div>

        <div className="flex justify-between space-x-4 pt-4 text-xl md:justify-start">
          {isExternalLink ? (
            <a href={buttonLink} target="_blank" rel="noreferrer">
              <Button variant="filled" rounded="large">
                <div className="px-1.5 py-0.5">{buttonText}</div>
              </Button>
            </a>
          ) : (
            <Link href={buttonLink} passHref>
              <div>
                <Button variant="filled" rounded="large">
                  <div className="px-1.5 py-0.5">{buttonText}</div>
                </Button>
              </div>
            </Link>
          )}
        </div>
      </div>
    </div>
  );
}
