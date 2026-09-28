import type { EdgeResponse } from '$lib/types/EdgeResponse';
import { ancestorPaths } from './paths';

/**
 * Nodes hidden inside a collapsed path → the collapsed path that stands in
 * for them: the outermost collapsed one, since everything below it is
 * hidden too.
 */
export function standIns(
	nodeIds: Iterable<string>,
	parentOf: Map<string, string>,
	collapsed: Set<string>
): Map<string, string> {
	const standIn = new Map<string, string>();
	for (const id of nodeIds) {
		const outermost = ancestorPaths(id, parentOf).findLast((path) => collapsed.has(path));
		if (outermost) standIn.set(id, outermost);
	}
	return standIn;
}

export type RoutedEdge = {
	/** The edge to draw: the first underlying edge, ends moved to their stand-ins. */
	edge: EdgeResponse;
	/** Every underlying edge drawn as this one line. */
	merged: EdgeResponse[];
	/** An end was moved onto a collapsed path. */
	rerouted: boolean;
};

/**
 * Connection lines for a canvas with collapsed paths: an edge touching a
 * hidden node is drawn from/to the collapsed path instead. Edges that end up
 * inside one collapsed path vanish, duplicates merge into one line, and
 * `part_of` is containment (the box), never a line.
 */
export function rerouteEdges(edges: EdgeResponse[], standIn: Map<string, string>): RoutedEdge[] {
	const routed = new Map<string, RoutedEdge>();
	for (const edge of edges) {
		if (edge.kind === 'part_of') continue;
		const from = standIn.get(edge.from_node_id) ?? edge.from_node_id;
		const to = standIn.get(edge.to_node_id) ?? edge.to_node_id;
		if (from === to) continue;
		const key = `${edge.kind}:${from}:${to}`;
		const existing = routed.get(key);
		if (existing) {
			existing.merged.push(edge);
			existing.rerouted = true;
			continue;
		}
		routed.set(key, {
			edge: { ...edge, from_node_id: from, to_node_id: to },
			merged: [edge],
			rerouted: from !== edge.from_node_id || to !== edge.to_node_id
		});
	}
	return [...routed.values()];
}
