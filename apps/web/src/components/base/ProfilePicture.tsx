interface Props {
  src: string;
}

export function ProfilePicture({ src }: Props) {
  return (
    <img
      src={src}
      alt="profile picture"
      className="object-cover rounded-full w-full h-full"
    />
  );
}
