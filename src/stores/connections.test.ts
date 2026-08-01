import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useConnectionsStore } from "@/stores/connections";

describe("connections store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("ouvre un onglet et le rend actif", () => {
    const store = useConnectionsStore();
    const id = store.open("vps", ["vps"]);
    expect(store.tabs).toHaveLength(1);
    expect(store.activeTabId).toBe(id);
    expect(store.activeTab?.panes).toHaveLength(1);
  });

  it("découpe l'onglet actif en deux panes (même cible)", () => {
    const store = useConnectionsStore();
    store.open("vps", ["vps"]);
    store.splitActive("row");
    expect(store.activeTab?.panes).toHaveLength(2);
    expect(store.activeTab?.direction).toBe("row");
    expect(store.activeTab?.panes[1].args).toEqual(["vps"]);
  });

  it("ferme un pane, puis l'onglet quand le dernier part", () => {
    const store = useConnectionsStore();
    const tabId = store.open("vps", ["vps"]);
    store.splitActive("column");
    const [p1] = store.activeTab!.panes;
    store.closePane(tabId, p1.id);
    expect(store.activeTab?.panes).toHaveLength(1);
    store.closePane(tabId, store.activeTab!.panes[0].id);
    expect(store.tabs).toHaveLength(0);
    expect(store.activeTabId).toBeNull();
  });

  it("sélectionne un voisin quand l'onglet actif est fermé", () => {
    const store = useConnectionsStore();
    const a = store.open("a", ["a"]);
    const b = store.open("b", ["b"]);
    expect(store.activeTabId).toBe(b);
    store.closeTab(b);
    expect(store.activeTabId).toBe(a);
  });

  it("compose un libellé d'onglet avec le nombre de splits", () => {
    const store = useConnectionsStore();
    store.open("vps", ["vps"]);
    expect(store.tabTitle(store.activeTab!)).toBe("vps");
    store.splitActive("row");
    expect(store.tabTitle(store.activeTab!)).toBe("vps +1");
  });
});
