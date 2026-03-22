import { describe, expect, it } from "vitest";
import {
  autoExploreAction,
  buyItemAction,
  clickMoveAction,
  craftAction,
  interactAction,
  levelUpAction,
  moveAction,
  sellItemAction,
  useAbilityAction,
  waitAction,
} from "./api";

describe("api action helpers", () => {
  it("creates directional move action", () => {
    expect(moveAction("NE")).toEqual({ action_type: { Move: "NE" } });
  });

  it("creates scalar and object actions consistently", () => {
    expect(waitAction()).toEqual({ action_type: "Wait" });
    expect(autoExploreAction()).toEqual({ action_type: "AutoExplore" });
    expect(interactAction()).toEqual({ action_type: "Interact" });
  });

  it("creates nested payload actions", () => {
    expect(clickMoveAction(4, 7)).toEqual({ action_type: { ClickMove: { x: 4, y: 7 } } });
    expect(levelUpAction("Attack")).toEqual({ action_type: { LevelUpChoice: "Attack" } });
    expect(buyItemAction(9, 1)).toEqual({ action_type: { BuyItem: { shop_id: 9, index: 1 } } });
    expect(sellItemAction(2, 9)).toEqual({ action_type: { SellItem: { index: 2, shop_id: 9 } } });
    expect(craftAction(3, 6)).toEqual({ action_type: { Craft: { weapon_idx: 3, scroll_idx: 6 } } });
  });

  it("normalizes nullable ability targets", () => {
    expect(useAbilityAction("fireball", { x: 2, y: 8 })).toEqual({
      action_type: { UseAbility: { ability_id: "fireball", target: { x: 2, y: 8 } } },
    });

    expect(useAbilityAction("blink")).toEqual({
      action_type: { UseAbility: { ability_id: "blink", target: null } },
    });
  });
});
