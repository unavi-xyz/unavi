import { render, screen } from "@testing-library/react";
import { AiFillHome } from "react-icons/ai";
import SidebarButton from "./SidebarButton";

it("should render text", () => {
  render(<SidebarButton icon="A" />);
  const button = screen.getByText("A");
  expect(button).toBeVisible();
});

it("should render images", () => {
  render(<SidebarButton image="https://picsum.photos/200" />);
  const button = screen.getByRole("img");

  expect(button).toBeVisible();
});

it("should render icons", () => {
  render(<SidebarButton icon={<AiFillHome />} />);
  const button = screen.getByRole("button");
  expect(button).toBeVisible();
});

it("should be a circle", () => {
  render(<SidebarButton />);
  const button = screen.getByRole("button");

  expect(button).toHaveClass("rounded-3xl");
});

it("should expand when selected", () => {
  render(<SidebarButton selected />);
  const button = screen.getByRole("button");

  expect(button).toHaveClass("rounded-xl");
});

it("should be light", () => {
  render(<SidebarButton />);
  const button = screen.getByRole("button");

  expect(button).toHaveClass("bg-white hover:bg-neutral-300 text-neutral-800");
});

it("should be dark", () => {
  render(<SidebarButton dark />);
  const button = screen.getByRole("button");

  expect(button).toHaveClass("bg-neutral-800 hover:bg-neutral-700 text-white");
});
