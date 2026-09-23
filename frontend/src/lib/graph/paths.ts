import type { EdgeResponse } from '$lib/types/EdgeResponse';

/**
 * Path containment from `part_of` edges: child id → the path it sits in. A
 * node sits in at most one path (enforced by the backend); paths can nest.
 */
export function parentPathOf(edges: EdgeResponse[]): Map<string, string> {
	return new Map(
		edges
			.filter((edge) => edge.kind === 'part_of')
			.map((edge) => [edge.from_node_id, edge.to_node_id])
	);
}

/** Every path id enclosing `nodeId`, innermost first. Cycle-safe. */
export function ancestorPaths(nodeId: string, parentOf: Map<string, string>): string[] {
	const ancestors: string[] = [];
	for (let parent = parentOf.get(nodeId); parent && !ancestors.includes(parent);) {
		ancestors.push(parent);
		parent = parentOf.get(parent);
	}
	return ancestors;
}

/** Nesting depth: 0 for top-level nodes, 1 inside a path, 2 inside a nested path... */
export function nestingDepth(nodeId: string, parentOf: Map<string, string>): number {
	return ancestorPaths(nodeId, parentOf).length;
}

/**
 * True when `node` matches, or is a path with a matching node somewhere
 * inside it (so a path stays visible while any of its contents match).
 */
export function matchesOrContainsMatch<Item extends { id: string; kind: string }>(
	node: Item,
	nodes: Item[],
	parentOf: Map<string, string>,
	matches: (node: Item) => boolean,
	seen = new Set<string>()
): boolean {
	if (matches(node)) return true;
	if (node.kind !== 'path' || seen.has(node.id)) return false;
	seen.add(node.id);
	return nodes.some(
		(child) =>
			parentOf.get(child.id) === node.id &&
			matchesOrContainsMatch(child, nodes, parentOf, matches, seen)
	);
}
