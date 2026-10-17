import { rerouteEdges } from '$lib/graph/collapse';
import { flowDirection, type CanvasPlacement } from '$lib/graph/layout';
import { nestingDepth } from '$lib/graph/paths';
import type { EdgeResponse } from '$lib/types/EdgeResponse';
import type { NodeResponse } from '$lib/types/NodeResponse';
import type { LoomCardNode, LoomGraphEdge } from './types';

/** xyflow nodes for laid-out `drawn` nodes; an expanded path is a box, the rest are cards. */
export function toFlowNodes(
	drawn: NodeResponse[],
	placements: Map<string, CanvasPlacement>,
	collapsed: Set<string>,
	parentOf: Map<string, string>,
	dimmedIds: Set<string>,
	selectedIds: Set<string> = new Set()
): LoomCardNode[] {
	// xyflow needs a parent listed before its children.
	const parentsFirst = drawn.toSorted(
		(left, right) => nestingDepth(left.id, parentOf) - nestingDepth(right.id, parentOf)
	);
	return parentsFirst.map((node) => {
		const { position, size, parentId } = placements.get(node.id)!;
		const isPath = node.kind === 'path' && !collapsed.has(node.id);
		return {
			id: node.id,
			type: isPath ? ('loomPath' as const) : ('loom' as const),
			position,
			parentId,
			// Path boxes have an explicit size (resizable); cards size themselves.
			...(isPath ? { width: size.width, height: size.height } : {}),
			data: { node, dimmed: dimmedIds.has(node.id) },
			selected: selectedIds.has(node.id),
			deletable: false
		};
	});
}

/**
 * xyflow edges between drawn nodes (`shownIds`). Connections into collapsed
 * paths are drawn to their stand-ins; part_of is drawn as containment (the
 * box), not as a line.
 */
export function toFlowEdges(
	edges: EdgeResponse[],
	standIn: Map<string, string>,
	shownIds: Set<string>,
	placements: Map<string, CanvasPlacement>,
	dimmedIds: Set<string>,
	nodeById: Map<string, NodeResponse>
): LoomGraphEdge[] {
	// Placements are relative to the parent path; walk up for the canvas y.
	const absoluteY = (id: string): number => {
		const placement = placements.get(id);
		if (!placement) return 0;
		return placement.position.y + (placement.parentId ? absoluteY(placement.parentId) : 0);
	};
	return rerouteEdges(edges, standIn)
		.filter(({ edge }) => shownIds.has(edge.from_node_id) && shownIds.has(edge.to_node_id))
		.map(({ edge, merged, rerouted }) => {
			let { source, target } = flowDirection(edge);
			const related = edge.kind === 'related';
			// related has no arrow: draw it from the upper card down so the
			// curve never loops back. ponytail: cards side by side at the same
			// y get an S-curve; compare |dx| to |dy| and fall back to the
			// left/right handles if that matters.
			if (related && absoluteY(source) > absoluteY(target)) [source, target] = [target, source];
			const unmet = merged.some(
				(underlying) =>
					underlying.kind === 'requires' && nodeById.get(underlying.to_node_id)?.status !== 'done'
			);
			return {
				id: edge.id,
				source,
				target,
				// A line standing in for hidden connections isn't one edge to delete.
				deletable: !rerouted,
				...(related
					? { sourceHandle: 'bottom', targetHandle: 'top' }
					: { sourceHandle: 'right', targetHandle: 'left' }),
				type: 'loom' as const,
				zIndex: 1,
				data: {
					kind: edge.kind,
					unmet,
					dimmed: dimmedIds.has(edge.from_node_id) || dimmedIds.has(edge.to_node_id)
				}
			};
		});
}
