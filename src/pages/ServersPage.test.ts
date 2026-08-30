import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { serversRepository } from "@/repositories/serversRepository";
import { nextTick } from "vue";
import { useSettingsStore } from "@/stores/settings";
import type { Host, Server, ServerItem } from "@/types/ssh";
import ServerCard from "@/components/ServerCard.vue";
import ServersPage from "./ServersPage.vue";

vi.mock("vue-router", () => ({ useRouter: () => ({ push: vi.fn() }) }));

function server(id: number, name: string, group: string | null): Server {
  return {
    id,
    name,
    hostname: `10.0.0.${id}`,
    port: 22,
    username: "root",
    identityFile: null,
    color: null,
    favorite: false,
    tags: [],
    group,
    os: null,
  };
}

const SERVERS: Server[] = [
  server(1, "web-1", "Production/Web"),
  server(2, "db-1", "Production"),
  server(3, "perso", null),
];

/** Un hôte du ~/.ssh/config rangé dans un dossier de la base : même arbre. */
const HOSTS: Host[] = [
  {
    alias: "bastion",
    hostname: "bastion.example",
    user: "ops",
    port: 22,
    identityFile: null,
    proxyJump: null,
    group: "Production/Web",
    color: null,
    tags: [],
    favorite: false,
    os: null,
    sourceFile: "~/.ssh/config",
  },
];

/** `Archives` n'a aucun serveur : seule la table `groups` le fait exister. */
const GROUP_PATHS = ["Archives", "Production", "Production/Web"];

vi.mock("@/repositories/serversRepository", () => ({
  serversRepository: {
    list: vi.fn(async () => SERVERS),
    create: vi.fn(),
    update: vi.fn(),
    remove: vi.fn(),
    setOs: vi.fn(),
  },
}));
vi.mock("@/repositories/hostsRepository", () => ({
  hostsRepository: {
    list: vi.fn(async () => HOSTS),
    create: vi.fn(),
    update: vi.fn(),
    remove: vi.fn(),
    setOs: vi.fn(),
  },
}));
vi.mock("@/repositories/groupsRepository", () => ({
  groupsRepository: {
    list: vi.fn(async () => GROUP_PATHS),
    create: vi.fn(async (path: string) => path),
    rename: vi.fn(),
    move: vi.fn(),
    remove: vi.fn(),
  },
}));

const stubs = {
  ServerCard: {
    props: ["item"],
    emits: ["dragstart", "dragend"],
    template: `<div class="stub-card" @dragstart="$emit('dragstart', item, $event)">{{
      item.name
    }}</div>`,
  },
  OsBadge: true,
  ServerFormDialog: true,
  GroupDialog: true,
};

async function render() {
  const wrapper = mount(ServersPage, { global: { stubs } });
  await flushPromises();
  return wrapper;
}

const texts = (wrapper: Awaited<ReturnType<typeof render>>, selector: string) =>
  wrapper.findAll(selector).map((node) => node.text());

/**
 * Node fournit un `localStorage` désactivé qui masque celui de jsdom : le store
 * des réglages y écrit dès qu'on change de vue. Un stub mémoire suffit.
 */
function stubLocalStorage(): void {
  const store = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => void store.set(key, value),
    removeItem: (key: string) => void store.delete(key),
    clear: () => store.clear(),
  });
}

describe("ServersPage — navigation par dossiers", () => {
  beforeEach(() => {
    stubLocalStorage();
    setActivePinia(createPinia());
  });

  it("n'affiche à la racine que les dossiers et les serveurs sans dossier", async () => {
    const wrapper = await render();

    expect(texts(wrapper, ".folder__name")).toEqual(["Archives", "Production"]);
    expect(texts(wrapper, ".stub-card")).toEqual(["perso"]);
  });

  it("compte les serveurs d'un dossier, sous-dossiers et config compris", async () => {
    const wrapper = await render();

    // Production = db-1 + web-1 + bastion (~/.ssh/config).
    const production = wrapper
      .findAll(".folder")
      .find((node) => node.text().startsWith("Production"));
    expect(production?.find(".folder__count").text()).toBe("3 serveurs");
  });

  it("donne au dossier le gabarit de la vue choisie", async () => {
    const wrapper = await render();
    const settings = useSettingsStore();

    settings.serversView = "grid";
    await nextTick();
    expect(wrapper.find(".folder").classes()).toContain("folder--tile");
    // La grille des dossiers est celle des cartes : mêmes colonnes.
    expect(wrapper.find(".folder").element.parentElement?.className).toBe("grid");

    settings.serversView = "list";
    await nextTick();
    expect(wrapper.find(".folder").classes()).toContain("folder--row");
    expect(wrapper.find(".folder").element.parentElement?.className).toBe("rows");
  });

  it("entre dans un dossier au clic et n'y montre que son contenu direct", async () => {
    const wrapper = await render();

    await wrapper
      .findAll(".folder")
      .find((node) => node.text().startsWith("Production"))!
      .trigger("click");

    expect(texts(wrapper, ".crumbs__item")).toEqual(["Serveurs", "Production"]);
    expect(texts(wrapper, ".folder__name")).toEqual(["Web"]);
    expect(texts(wrapper, ".stub-card")).toEqual(["db-1"]);
  });

  it("descend dans un sous-dossier et y mêle les deux origines", async () => {
    const wrapper = await render();

    await wrapper
      .findAll(".folder")
      .find((node) => node.text().startsWith("Production"))!
      .trigger("click");
    await wrapper.find(".folder").trigger("click");

    expect(texts(wrapper, ".crumbs__item")).toEqual(["Serveurs", "Production", "Web"]);
    expect(texts(wrapper, ".stub-card").sort()).toEqual(["bastion", "web-1"]);
  });

  it("remonte par le fil d'Ariane", async () => {
    const wrapper = await render();

    await wrapper
      .findAll(".folder")
      .find((node) => node.text().startsWith("Production"))!
      .trigger("click");
    await wrapper.find(".crumbs__item").trigger("click");

    expect(texts(wrapper, ".crumbs__item")).toEqual(["Serveurs"]);
    expect(texts(wrapper, ".stub-card")).toEqual(["perso"]);
  });

  it("cherche dans toute l'arborescence, pas seulement le dossier ouvert", async () => {
    const wrapper = await render();

    // Depuis « Archives », donc un dossier vide : le résultat vient d'ailleurs.
    await wrapper
      .findAll(".folder")
      .find((node) => node.text().startsWith("Archives"))!
      .trigger("click");
    await wrapper.find(".search input").setValue("web-1");

    expect(texts(wrapper, ".stub-card")).toEqual(["web-1"]);
    // Le dossier d'origine est rappelé au-dessus du résultat.
    expect(texts(wrapper, ".group__name")).toEqual(["Production/Web"]);
  });

  it("annonce un dossier vide sans proposer deux fois d'y ajouter", async () => {
    const wrapper = await render();

    await wrapper
      .findAll(".folder")
      .find((node) => node.text().startsWith("Archives"))!
      .trigger("click");

    expect(wrapper.find(".empty").text()).toContain("Ce dossier est vide");
    expect(wrapper.find(".addcard").exists()).toBe(false);
  });
});

describe("ServersPage — glisser un serveur dans un dossier", () => {
  beforeEach(() => {
    stubLocalStorage();
    setActivePinia(createPinia());
    vi.mocked(serversRepository.update).mockClear();
  });

  /** jsdom n'implémente pas `DataTransfer` : un mock suffit aux poignées. */
  const dataTransfer = () => ({
    setData: vi.fn(),
    effectAllowed: "",
    dropEffect: "",
  });

  const findFolder = (wrapper: Awaited<ReturnType<typeof render>>, name: string) =>
    wrapper.findAll(".folder").find((node) => node.text().startsWith(name))!;

  it("rend les lignes saisissables en vue liste", async () => {
    const wrapper = await render();
    const settings = useSettingsStore();

    settings.serversView = "list";
    await nextTick();
    expect(wrapper.find(".row").attributes("draggable")).toBe("true");
  });

  it("rend la carte saisissable et désigne le serveur glissé", () => {
    // La vraie carte, sans stub : c'est elle qui porte `draggable` en vue grille.
    const item: ServerItem = {
      key: "local:1",
      source: "local",
      id: 1,
      alias: "web-1",
      name: "web-1",
      hostname: "10.0.0.1",
      port: 22,
      username: "root",
      identityFile: null,
      color: null,
      favorite: false,
      tags: [],
      group: "Production/Web",
      os: null,
      sourceFile: null,
    };
    const card = mount(ServerCard, {
      props: { item },
      global: { stubs: { OsBadge: true } },
    });

    expect(card.find(".tile").attributes("draggable")).toBe("true");
    card.find(".tile").trigger("dragstart");
    // Vue passe le prop via un proxy réactif : on compare le contenu.
    expect(card.emitted("dragstart")?.[0]?.[0]).toEqual(item);
  });

  it("range le serveur déposé dans le dossier visé", async () => {
    const wrapper = await render();
    const dt = dataTransfer();

    await wrapper.find(".stub-card").trigger("dragstart", { dataTransfer: dt });
    const production = findFolder(wrapper, "Production");
    await production.trigger("dragover", { dataTransfer: dt });
    await production.trigger("drop", { dataTransfer: dt });
    await flushPromises();

    // « perso » était hors dossier : il rejoint Production.
    expect(vi.mocked(serversRepository.update)).toHaveBeenCalledWith(
      3,
      expect.objectContaining({ name: "perso", group: "Production" }),
    );
  });

  it("ne surligne que le dossier survolé", async () => {
    const wrapper = await render();
    const dt = dataTransfer();

    await wrapper.find(".stub-card").trigger("dragstart", { dataTransfer: dt });
    await findFolder(wrapper, "Production").trigger("dragover", { dataTransfer: dt });

    expect(findFolder(wrapper, "Production").classes()).toContain("folder--over");
    expect(findFolder(wrapper, "Archives").classes()).not.toContain("folder--over");
  });

  it("sort un serveur de son dossier en le déposant sur le fil d'Ariane", async () => {
    const wrapper = await render();

    await findFolder(wrapper, "Production").trigger("click");
    const dt = dataTransfer();
    await wrapper.find(".stub-card").trigger("dragstart", { dataTransfer: dt });

    const racine = wrapper.findAll(".crumbs__item")[0];
    await racine.trigger("dragover", { dataTransfer: dt });
    expect(racine.classes()).toContain("crumbs__item--over");
    await racine.trigger("drop", { dataTransfer: dt });
    await flushPromises();

    expect(vi.mocked(serversRepository.update)).toHaveBeenCalledWith(
      2,
      expect.objectContaining({ name: "db-1", group: null }),
    );
  });

  it("ignore le dépôt là où le serveur se trouve déjà", async () => {
    const wrapper = await render();
    const dt = dataTransfer();

    // « perso » est déjà à la racine : le fil d'Ariane ne doit rien accepter.
    await wrapper.find(".stub-card").trigger("dragstart", { dataTransfer: dt });
    const racine = wrapper.findAll(".crumbs__item")[0];
    await racine.trigger("dragover", { dataTransfer: dt });
    await racine.trigger("drop", { dataTransfer: dt });
    await flushPromises();

    expect(racine.classes()).not.toContain("crumbs__item--over");
    expect(vi.mocked(serversRepository.update)).not.toHaveBeenCalled();
  });

  it("oublie le glissement une fois lâché", async () => {
    const wrapper = await render();
    const dt = dataTransfer();

    await wrapper.find(".stub-card").trigger("dragstart", { dataTransfer: dt });
    const production = findFolder(wrapper, "Production");
    await production.trigger("dragover", { dataTransfer: dt });
    await production.trigger("drop", { dataTransfer: dt });
    await flushPromises();

    // Plus rien n'est en cours : survoler à nouveau ne doit pas surligner.
    await findFolder(wrapper, "Archives").trigger("dragover", { dataTransfer: dt });
    expect(findFolder(wrapper, "Archives").classes()).not.toContain("folder--over");
  });
});
