import { describe, expect, it } from "vitest";
import { compareVersions } from "./updateCheck";

describe("compareVersions", () => {
  it("ordonne les versions par composant", () => {
    expect(compareVersions("0.3.0", "0.2.0")).toBeGreaterThan(0);
    expect(compareVersions("0.2.0", "0.3.0")).toBeLessThan(0);
    expect(compareVersions("0.2.0", "0.2.0")).toBe(0);
  });

  it("compare numériquement et non alphabétiquement", () => {
    expect(compareVersions("0.10.0", "0.9.0")).toBeGreaterThan(0);
  });

  it("tolère le préfixe v et les composants manquants", () => {
    expect(compareVersions("v1.0.0", "1")).toBe(0);
    expect(compareVersions("v0.2.1", "0.2")).toBeGreaterThan(0);
  });
});
