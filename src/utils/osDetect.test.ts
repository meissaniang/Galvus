import { describe, expect, it } from "vitest";
import { BannerCollector, detectOs } from "./osDetect";

const UBUNTU_BANNER = `Welcome to Ubuntu 22.04.5 LTS (GNU/Linux 6.8.0-1057-oracle aarch64)

 * Documentation:  https://help.ubuntu.com
 * Management:     https://landscape.canonical.com
 * Support:        https://ubuntu.com/pro
`;

describe("detectOs", () => {
  it("reconnaît Ubuntu dans une bannière réelle", () => {
    expect(detectOs(UBUNTU_BANNER)).toBe("ubuntu");
  });

  it("préfère la distribution la plus spécifique à « linux »", () => {
    expect(detectOs("Debian GNU/Linux 12 (bookworm)")).toBe("debian");
    expect(detectOs("Welcome to Linux Mint 21")).toBe("mint");
    expect(detectOs("Rocky Linux 9.3 (Blue Onyx)")).toBe("rocky");
  });

  it("retombe sur Linux générique quand la distribution est inconnue", () => {
    expect(detectOs("Linux srv01 6.1.0 x86_64 GNU/Linux")).toBe("linux");
  });

  it("ne devine rien sur une bannière muette", () => {
    expect(detectOs("Last login: Sat Aug 8 14:54:22 2026")).toBeNull();
  });

  it("voit à travers les séquences de couleur", () => {
    expect(detectOs("\x1b[1;32mWelcome to Alpine Linux\x1b[0m")).toBe("alpine");
  });
});

describe("BannerCollector", () => {
  const encode = (s: string) => new TextEncoder().encode(s);

  it("reconnaît le système à cheval sur deux morceaux", () => {
    const collector = new BannerCollector();
    expect(collector.push(encode("Welcome to Ubu"))).toBeNull();
    expect(collector.push(encode("ntu 24.04 LTS"))).toBe("ubuntu");
  });

  it("ne signale qu'une fois, puis se tait", () => {
    const collector = new BannerCollector();
    expect(collector.push(encode("Debian GNU/Linux 12"))).toBe("debian");
    expect(collector.push(encode("Welcome to Ubuntu"))).toBeNull();
  });

  it("abandonne passé la bannière, pour ne pas lire la sortie des commandes", () => {
    const collector = new BannerCollector(32);
    expect(collector.push(encode("x".repeat(40)))).toBeNull();
    expect(collector.push(encode("Welcome to Ubuntu"))).toBeNull();
  });
});
