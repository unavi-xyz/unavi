/**
 * @jest-environment jsdom
 */
import { Engine } from "../Engine";
import Worker from "../__mocks__/Worker";

it("should create an instance of Engine", () => {
  window.Worker = Worker as any;

  expect(
    new Engine({
      canvas: document.createElement("canvas"),
    })
  ).not.toBeUndefined();
});
