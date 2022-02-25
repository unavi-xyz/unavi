import { render, screen } from "@testing-library/react";
import Dialog from "./Dialog";

const mockSet = jest.fn();

it("should be visible when open", () => {
  render(
    <Dialog open setOpen={mockSet}>
      <></>
    </Dialog>
  );
  const dialog = screen.getByRole("dialog");

  expect(dialog).toBeVisible();
});

it("should be unmounted when closed", () => {
  render(
    <Dialog open={false} setOpen={mockSet}>
      <></>
    </Dialog>
  );
  const dialog = screen.queryByRole("dialog");

  expect(dialog).toBeNull();
});

it("should show children", () => {
  render(
    <Dialog open setOpen={mockSet}>
      <p>Hello!</p>
    </Dialog>
  );
  const text = screen.getByText("Hello!");

  expect(text).toBeVisible();
});
