import { Dispatch, SetStateAction } from "react";
import { TextField } from "@mui/material";

interface Props {
  step?: number;
  max?: number;
  min?: number;
  value: number;
  setValue: Dispatch<SetStateAction<number>>;
}

export default function NumberField({
  step = 0.1,
  max,
  min,
  value,
  setValue,
}: Props) {
  function handleChange(e) {
    setValue(Number(e.target.value));
  }

  const rounded = Math.round(value * 100) / 100;

  return (
    <TextField
      value={rounded}
      onChange={handleChange}
      type="number"
      variant="standard"
      fullWidth
      inputProps={{ step, max, min }}
    />
  );
}
