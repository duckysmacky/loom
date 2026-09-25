import type { EdgeResponse } from '$lib/types/EdgeResponse';
import type { NodeResponse } from '$lib/types/NodeResponse';

/**
 * Reorders `nodes` so every node comes after the nodes it `requires` (among
 * those in the same list), so a blocked card sits right after its blocker and
 * a tier reads as a sequence. Otherwise keeps the incoming order; cycle-safe.
 */
export function dependencyOrder(nodes: NodeResponse[], edges: EdgeResponse[]): NodeResponse[] {
	const indexById = new Map(nodes.map((node, index) => [node.id, index]));
	const prerequisites = new Map<string, number[]>();
	for (const edge of edges) {
		const prerequisite = indexById.get(edge.to_node_id);
		if (edge.kind !== 'requires' || prerequisite === undefined) continue;
		if (!indexById.has(edge.from_node_id)) continue;
		const list = prerequisites.get(edge.from_node_id) ?? [];
		list.push(prerequisite);
		prerequisites.set(edge.from_node_id, list);
	}

	const ordered: NodeResponse[] = [];
	const visited = new Set<string>();
	const visit = (node: NodeResponse) => {
		if (visited.has(node.id)) return;
		visited.add(node.id);
		const before = (prerequisites.get(node.id) ?? []).toSorted((left, right) => left - right);
		for (const index of before) visit(nodes[index]);
		ordered.push(node);
	};
	nodes.forEach(visit);
	return ordered;
}

/**
 * Applies manual drag order on top of the auto order: ranked nodes
 * (`sort_order` set) sort by rank first, then every unranked node keeps
 * its incoming (auto) order, appended after them.
 */
export function manualOrder(nodes: NodeResponse[]): NodeResponse[] {
	const ranked = nodes
		.filter((node) => node.sort_order !== null)
		.toSorted((left, right) => left.sort_order! - right.sort_order!);
	const unranked = nodes.filter((node) => node.sort_order === null);
	return [...ranked, ...unranked];
}

/**
 * Reinserts `draggedId` into `ids` right before `beforeId` (or at the end
 * when `beforeId` is null/missing), for the caller to send as the new
 * manual rank of that group via `nodesApi.reorder`.
 */
export function moveBefore(ids: string[], draggedId: string, beforeId: string | null): string[] {
	const rest = ids.filter((id) => id !== draggedId);
	const index = beforeId ? rest.indexOf(beforeId) : -1;
	if (index === -1) return [...rest, draggedId];
	return [...rest.slice(0, index), draggedId, ...rest.slice(index)];
}
