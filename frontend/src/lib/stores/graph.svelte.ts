import { boardApi, topicsApi } from '$lib/api/endpoints';
import type { EdgeResponse } from '$lib/types/EdgeResponse';
import type { NodeResponse } from '$lib/types/NodeResponse';
import type { TopicResponse } from '$lib/types/TopicResponse';
import { notifyError } from './toasts.svelte';

/**
 * Client-side cache of the signed-in user's whole graph (`/board/canvas` +
 * `/topics`). Every board view, the detail panel, the command palette and
 * link lookups read from it. After any mutation the whole graph is refetched:
 * `blocked`/`container_progress` are derived server-side and one edit can
 * change them on other nodes. `version` bumps on every load so pages backed
 * by their own endpoint (dashboard, nodes list) know to refetch too.
 */
class GraphStore {
	nodes = $state<NodeResponse[]>([]);
	edges = $state<EdgeResponse[]>([]);
	topics = $state<TopicResponse[]>([]);
	loaded = $state(false);
	version = $state(0);

	nodeById = $derived(new Map(this.nodes.map((node) => [node.id, node])));
	topicById = $derived(new Map(this.topics.map((topic) => [topic.id, topic])));

	async load() {
		try {
			const [canvas, topics] = await Promise.all([boardApi.canvas(), topicsApi.list()]);
			this.nodes = canvas.nodes;
			this.edges = canvas.edges;
			this.topics = topics;
			this.loaded = true;
			this.version++;
		} catch (error) {
			notifyError(error);
		}
	}

	/**
	 * Runs an API mutation, then refreshes the graph. Errors are surfaced as a
	 * toast and resolve to `undefined`, so callers never need their own catch.
	 */
	async mutate<T>(action: () => Promise<T>): Promise<T | undefined> {
		try {
			const result = await action();
			await this.load();
			return result;
		} catch (error) {
			notifyError(error);
			return undefined;
		}
	}

	reset() {
		this.nodes = [];
		this.edges = [];
		this.topics = [];
		this.loaded = false;
	}
}

export const graph = new GraphStore();
