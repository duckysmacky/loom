import dagre from '@dagrejs/dagre';
import type { EdgeResponse } from '$lib/types/EdgeResponse';
import type { NodeResponse } from '$lib/types/NodeResponse';

export const CANVAS_NODE_WIDTH = 190;
export const CANVAS_NODE_HEIGHT = 96;
const GAP = 80;

export type Point = { x: number; y: number };

/**
 * Canvas edges always point prerequisite → dependent / child → container, so
 * the graph reads left to right: `requires` is stored dependent → prerequisite
 * and is flipped here; `part_of` and `related` keep their stored direction.
 */
export function flowDirection(edge: EdgeResponse): { source: string; target: string } {
	return edge.kind === 'requires'
		? { source: edge.to_node_id, target: edge.from_node_id }
		: { source: edge.from_node_id, target: edge.to_node_id };
}

/**
 * Canvas position for every node. Nodes the user has placed keep their saved
 * position; the rest are laid out left-to-right by dagre (among themselves)
 * and parked below everything already placed, so they never land on top of
 * a hand-placed node.
 */
export function layoutPositions(nodes: NodeResponse[], edges: EdgeResponse[]): Map<string, Point> {
	const positions = new Map<string, Point>();
	const unplaced = nodes.filter((node) => node.canvas_x === null || node.canvas_y === null);
	let placedBottom = -Infinity;
	let placedLeft = Infinity;

	for (const node of nodes) {
		if (node.canvas_x === null || node.canvas_y === null) continue;
		positions.set(node.id, { x: node.canvas_x, y: node.canvas_y });
		placedBottom = Math.max(placedBottom, node.canvas_y + CANVAS_NODE_HEIGHT);
		placedLeft = Math.min(placedLeft, node.canvas_x);
	}
	if (!unplaced.length) return positions;

	const layoutGraph = new dagre.graphlib.Graph();
	layoutGraph.setGraph({ rankdir: 'LR', nodesep: 40, ranksep: GAP });
	layoutGraph.setDefaultEdgeLabel(() => ({}));
	const unplacedIds = new Set(unplaced.map((node) => node.id));
	for (const node of unplaced) {
		layoutGraph.setNode(node.id, { width: CANVAS_NODE_WIDTH, height: CANVAS_NODE_HEIGHT });
	}
	for (const edge of edges) {
		const { source, target } = flowDirection(edge);
		if (unplacedIds.has(source) && unplacedIds.has(target)) layoutGraph.setEdge(source, target);
	}
	dagre.layout(layoutGraph);

	// dagre reports centres; shift the block below the placed nodes, if any.
	const offsetX = Number.isFinite(placedLeft) ? placedLeft : 0;
	const offsetY = Number.isFinite(placedBottom) ? placedBottom + GAP : 0;
	for (const node of unplaced) {
		const { x, y } = layoutGraph.node(node.id);
		positions.set(node.id, {
			x: x - CANVAS_NODE_WIDTH / 2 + offsetX,
			y: y - CANVAS_NODE_HEIGHT / 2 + offsetY
		});
	}
	return positions;
}
