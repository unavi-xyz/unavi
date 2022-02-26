import { fireEvent, render, screen } from "@testing-library/react";
import TextField from "./TextField";

it("should change ref value when typing in field", () => {
  const testRef = {
    current: null,
  };

  render(<TextField title="test" inputRef={testRef} />);
  const field = screen.getByPlaceholderText("test");

  fireEvent.change(field, { target: { value: "Hello!" } });

  expect(testRef.current.value).toEqual("Hello!");
});
