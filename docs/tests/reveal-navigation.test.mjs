import assert from "node:assert/strict";
import { test } from "node:test";
import { revealNavigation } from "../app/utils/reveal-navigation.ts";

function fixture({
  top = 10,
  bottom = 30,
  pathname = "/components/text",
  height = 100,
  visible = true,
} = {}) {
  const calls = [];
  const body = {
    clientHeight: height,
    clientTop: 0,
    scrollTop: 40,
    getBoundingClientRect: () => ({ top: 0 }),
    scrollTo: (options) => calls.push(options),
  };
  const active = {
    closest: () => ({ pathname }),
    getBoundingClientRect: () => ({ top, bottom }),
  };
  const navigation = {
    querySelector: (selector) =>
      selector === ".akaza-sidebar-body" ? body : active,
    querySelectorAll: () => [active],
    getClientRects: () => (visible ? [{}] : []),
  };
  return { navigation, body, calls };
}

test("offscreen active items move only the sidebar by the nearest offset", () => {
  const below = fixture({ top: 180, bottom: 210 });
  assert(revealNavigation(below.navigation, "/components/text/"));
  assert.deepEqual(below.calls, [{ top: 150, behavior: "instant" }]);
  const above = fixture({ top: -15, bottom: 5 });
  assert(revealNavigation(above.navigation, "/components/text"));
  assert.deepEqual(above.calls, [{ top: 25, behavior: "instant" }]);
});

test("visible, hidden and stale-route items do not trigger unnecessary scrolling", () => {
  const inside = fixture();
  assert(revealNavigation(inside.navigation, "/components/text"));
  assert.deepEqual(inside.calls, []);
  for (const options of [
    { visible: false },
    { height: 0 },
    { pathname: "/components/button" },
  ]) {
    const { navigation, calls } = fixture(options);
    assert.equal(revealNavigation(navigation, "/components/text"), false);
    assert.deepEqual(calls, []);
  }
});

test("mobile navigation can match a link before selection attributes appear", () => {
  const { navigation, calls } = fixture({ top: 170, bottom: 200 });
  const original = navigation.querySelector;
  navigation.querySelector = (selector) =>
    selector === '[aria-current="page"]' ? null : original(selector);
  assert(revealNavigation(navigation, "/components/text/"));
  assert.deepEqual(calls, [{ top: 140, behavior: "instant" }]);
});
