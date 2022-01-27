import { Dispatch, SetStateAction } from "react";
import { Box, FormControl, MenuItem, Select } from "@mui/material";

interface Props {
  options: string[];
  value: string;
  setValue: Dispatch<SetStateAction<string>>;
}

export default function SelectField({ options, value, setValue }: Props) {
  function handleChange(e) {
    setValue(e.target.value);
  }

  return (
    <Box sx={{ minWidth: 120 }}>
      <FormControl fullWidth>
        <Select variant="standard" value={value} onChange={handleChange}>
          {options.map((option) => {
            return (
              <MenuItem key={option} value={option}>
                {option}
              </MenuItem>
            );
          })}
        </Select>
      </FormControl>
    </Box>
  );
}
