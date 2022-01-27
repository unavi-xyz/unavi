import { Dispatch, SetStateAction } from "react";
import { Checkbox } from "@mui/material";

interface Props {
  value: boolean;
  setValue: Dispatch<SetStateAction<boolean>>;
}

export default function CheckField({ value, setValue }: Props) {
  function handleChange(e) {
    setValue(Boolean(e.target.checked));
  }

  return <Checkbox checked={value} onChange={handleChange} />;
}
