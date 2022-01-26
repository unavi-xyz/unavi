import { Dispatch, SetStateAction, useEffect, useRef } from "react";
import { TextField } from "@mui/material";

interface Props {
  value: string;
  setValue: Dispatch<SetStateAction<string>>;
}

export default function ColorField({ value, setValue }: Props) {
  const ref = useRef<null | HTMLInputElement>();

  useEffect(() => {
    ref.current.value = value;
  }, [value]);

  useEffect(() => {
    const interval = setInterval(() => {
      setValue(ref.current.value);
    }, 100);

    return () => {
      clearInterval(interval);
    };
  }, [setValue]);

  return (
    <TextField
      inputRef={ref}
      id="inspect-input-color"
      type="color"
      variant="standard"
      fullWidth
      InputProps={{ disableUnderline: true }}
    />
  );
}
