import { Dispatch, SetStateAction, useEffect, useRef } from "react";
import { TextField } from "@mui/material";

interface Props {
  value: string;
  setValue: Dispatch<SetStateAction<string>>;
}

export default function ColorField({ value, setValue }: Props) {
  const ref = useRef(value);

  function handleChange(e) {
    ref.current = e.target.value;
  }

  useEffect(() => {
    const interval = setInterval(() => {
      setValue(ref.current);
    }, 100);

    return () => {
      clearInterval(interval);
    };
  }, [setValue]);

  return (
    <TextField
      onChange={handleChange}
      type="color"
      variant="standard"
      fullWidth
      InputProps={{ disableUnderline: true }}
    />
  );
}
