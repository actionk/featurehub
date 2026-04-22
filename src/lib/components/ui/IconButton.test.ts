import { describe, it, expect } from "vitest";
import { render, fireEvent } from "@testing-library/svelte";
import IconButtonTestWrapper from "./IconButtonTestWrapper.svelte";

describe("IconButton", () => {
  it("renders with default props", () => {
    const { container } = render(IconButtonTestWrapper);
    const button = container.querySelector("button");
    expect(button).toBeTruthy();
    expect(button!.classList.contains("icon-btn")).toBe(true);
    expect(button!.classList.contains("icon-btn--sm")).toBe(true);
    expect(button!.classList.contains("icon-btn--ghost")).toBe(true);
  });

  it("applies size and variant classes", () => {
    const { container } = render(IconButtonTestWrapper, {
      props: { size: "md", variant: "accent" },
    });
    const button = container.querySelector("button");
    expect(button!.classList.contains("icon-btn--md")).toBe(true);
    expect(button!.classList.contains("icon-btn--accent")).toBe(true);
  });

  it("sets disabled attribute", () => {
    const { container } = render(IconButtonTestWrapper, {
      props: { disabled: true },
    });
    const button = container.querySelector("button") as HTMLButtonElement;
    expect(button.disabled).toBe(true);
  });

  it("sets title attribute", () => {
    const { container } = render(IconButtonTestWrapper, {
      props: { title: "Click me" },
    });
    const button = container.querySelector("button");
    expect(button!.getAttribute("title")).toBe("Click me");
  });

  it("renders children content", () => {
    const { container } = render(IconButtonTestWrapper, {
      props: { label: "Save" },
    });
    const button = container.querySelector("button");
    expect(button!.textContent?.trim()).toBe("Save");
  });
});
