import { describe, expect, it } from "vitest";
import {
  ancestorsOf,
  depthOf,
  isWithin,
  joinPath,
  nameOf,
  normalizePath,
  parentOf,
  segments,
} from "./groupPath";

describe("groupPath", () => {
  it("découpe et nettoie un chemin", () => {
    expect(segments(" Prod / Web ")).toEqual(["Prod", "Web"]);
    expect(segments("Prod//Web")).toEqual(["Prod", "Web"]);
    expect(segments("   ")).toEqual([]);
  });

  it("recompose un chemin depuis des morceaux", () => {
    expect(joinPath("Prod", "Web")).toBe("Prod/Web");
    expect(joinPath("", "Prod")).toBe("Prod");
    expect(joinPath("Prod/Europe", "Web")).toBe("Prod/Europe/Web");
    expect(joinPath(null, undefined)).toBe("");
  });

  it("normalise un groupe absent en racine", () => {
    expect(normalizePath(null)).toBe("");
    expect(normalizePath(" Prod ")).toBe("Prod");
  });

  it("remonte au parent, jusqu'à la racine", () => {
    expect(parentOf("Prod/Europe/Web")).toBe("Prod/Europe");
    expect(parentOf("Prod")).toBe("");
    expect(parentOf("")).toBe("");
  });

  it("nomme le dossier par son dernier segment", () => {
    expect(nameOf("Prod/Europe/Web")).toBe("Web");
    expect(nameOf("")).toBe("");
  });

  it("énumère les ancêtres de la racine vers la feuille", () => {
    expect(ancestorsOf("Prod/Europe/Web")).toEqual([
      "Prod",
      "Prod/Europe",
      "Prod/Europe/Web",
    ]);
    expect(ancestorsOf("")).toEqual([]);
  });

  it("ne confond pas un préfixe textuel avec un descendant", () => {
    expect(isWithin("Prod/Web", "Prod")).toBe(true);
    expect(isWithin("Prod", "Prod")).toBe(true);
    expect(isWithin("Prodige", "Prod")).toBe(false);
    // Tout descend de la racine.
    expect(isWithin("Prod/Web", "")).toBe(true);
  });

  it("mesure la profondeur", () => {
    expect(depthOf("")).toBe(0);
    expect(depthOf("Prod/Web")).toBe(2);
  });
});
