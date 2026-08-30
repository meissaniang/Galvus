import { ref } from "vue";
import { defineStore } from "pinia";
import { groupsRepository } from "@/repositories/groupsRepository";
import { hostsRepository } from "@/repositories/hostsRepository";
import { useServersStore } from "@/stores/servers";
import { useMyServersStore } from "@/stores/myServers";
import { isWithin, parentOf } from "@/utils/groupPath";
import type { ConfigHostInput, Host } from "@/types/ssh";

/**
 * Dossiers de l'écran Serveurs.
 *
 * La table `groups` ne connaît que les serveurs de la base chiffrée. Les hôtes
 * du `~/.ssh/config` rangent leur dossier dans un commentaire de fichier :
 * renommer, déplacer ou supprimer un dossier doit donc les réécrire ici, à la
 * suite de l'opération SQL. C'est le rôle de ce store — le seul endroit d'où
 * une hiérarchie de dossiers est modifiée.
 */
export const useGroupsStore = defineStore("groups", () => {
  /** Chemins des dossiers créés explicitement (les dossiers vides compris). */
  const paths = ref<string[]>([]);
  const error = ref<string | null>(null);

  async function load(): Promise<void> {
    error.value = null;
    try {
      paths.value = await groupsRepository.list();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  function toInput(host: Host, group: string | null): ConfigHostInput {
    return {
      alias: host.alias,
      hostname: host.hostname,
      user: host.user,
      port: host.port,
      identityFile: host.identityFile,
      proxyJump: host.proxyJump,
      group,
      color: host.color,
      tags: host.tags,
      favorite: host.favorite,
      os: host.os,
    };
  }

  /**
   * Réécrit le dossier des hôtes de config rangés dans `path` ou sous lui.
   *
   * Les échecs sont ignorés hôte par hôte : un fichier `Include` en lecture
   * seule ne doit pas laisser l'opération à moitié faite sur les autres.
   */
  async function retargetConfigHosts(
    path: string,
    rewrite: (group: string) => string | null,
  ): Promise<void> {
    const hostsStore = useServersStore();
    const affected = hostsStore.hosts.filter(
      (host): host is Host & { group: string } =>
        Boolean(host.group) && isWithin(host.group as string, path),
    );
    for (const host of affected) {
      try {
        await hostsRepository.update(host.alias, toInput(host, rewrite(host.group)));
      } catch {
        // Signalé par le rechargement : l'hôte gardera son ancien dossier.
      }
    }
  }

  /** Recharge tout ce qu'une réorganisation a pu déplacer. */
  async function reloadAll(): Promise<void> {
    await Promise.all([load(), useMyServersStore().load(), useServersStore().load()]);
  }

  /** Crée un dossier (et ses parents). Rend son chemin normalisé. */
  async function create(path: string): Promise<string> {
    error.value = null;
    try {
      const created = await groupsRepository.create(path);
      await load();
      return created;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  /**
   * Fait exister un dossier saisi à la main dans le formulaire d'un serveur.
   *
   * Silencieux : le serveur est déjà enregistré à ce stade, et son dossier
   * apparaîtrait de toute façon dans l'arborescence, déduit de son chemin.
   */
  async function ensure(path: string | null): Promise<void> {
    if (!path || paths.value.includes(path)) return;
    try {
      await create(path);
    } catch {
      // Sans conséquence sur l'affichage.
    }
  }

  /** Renomme le dernier segment d'un dossier. Rend son nouveau chemin. */
  async function rename(path: string, name: string): Promise<string> {
    error.value = null;
    try {
      const renamed = await groupsRepository.rename(path, name);
      await retargetConfigHosts(path, (group) => renamed + group.slice(path.length));
      await reloadAll();
      return renamed;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  /** Déplace un dossier sous `parent` (`""` = racine). Rend son nouveau chemin. */
  async function move(path: string, parent: string): Promise<string> {
    error.value = null;
    try {
      const moved = await groupsRepository.move(path, parent);
      await retargetConfigHosts(path, (group) => moved + group.slice(path.length));
      await reloadAll();
      return moved;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  /**
   * Supprime un dossier et ses sous-dossiers.
   *
   * Les serveurs qu'ils contenaient remontent au dossier parent : supprimer un
   * rangement ne doit jamais emporter ce qui était rangé dedans.
   */
  async function remove(path: string): Promise<void> {
    error.value = null;
    const parent = parentOf(path);
    try {
      await groupsRepository.remove(path);
      await retargetConfigHosts(path, () => parent || null);
      await reloadAll();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  return { paths, error, load, create, ensure, rename, move, remove };
});
