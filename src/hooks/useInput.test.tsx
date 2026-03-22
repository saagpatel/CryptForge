import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useInput, type InputMode } from "./useInput";

function Harness({ mode, enabled, actions }: { mode: InputMode; enabled: boolean; actions: Parameters<typeof useInput>[1] }) {
  useInput(mode, actions, enabled);
  return null;
}

function createActions() {
  return {
    onMove: vi.fn(),
    onWait: vi.fn(),
    onPickUp: vi.fn(),
    onUseStairs: vi.fn(),
    onUseItem: vi.fn(),
    onDropItem: vi.fn(),
    onEquipItem: vi.fn(),
    onUnequipSlot: vi.fn(),
    onLevelUpChoice: vi.fn(),
    onToggleInventory: vi.fn(),
    onToggleInspect: vi.fn(),
    onEscape: vi.fn(),
    onInteract: vi.fn(),
    onAutoExplore: vi.fn(),
    onEnterTargeting: vi.fn(),
    onTargetMove: vi.fn(),
    onTargetCycleNext: vi.fn(),
    onTargetConfirm: vi.fn(),
    onUseAbility: vi.fn(),
  };
}

describe("useInput", () => {
  it("routes movement keys in normal mode", () => {
    const actions = createActions();
    render(<Harness mode="normal" enabled={true} actions={actions} />);

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "w" }));
    expect(actions.onMove).toHaveBeenCalledWith("N");
  });

  it("routes wait key in normal mode", () => {
    const actions = createActions();
    render(<Harness mode="normal" enabled={true} actions={actions} />);

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "." }));
    expect(actions.onWait).toHaveBeenCalled();
  });

  it("routes escape globally", () => {
    const actions = createActions();
    render(<Harness mode="normal" enabled={true} actions={actions} />);

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(actions.onEscape).toHaveBeenCalled();
  });

  it("does not process input when disabled", () => {
    const actions = createActions();
    render(<Harness mode="normal" enabled={false} actions={actions} />);

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "w" }));
    expect(actions.onMove).not.toHaveBeenCalled();
  });
});
