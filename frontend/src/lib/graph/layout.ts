import dagre from '@dagrejs/dagre';
import type { EdgeResponse } from '$lib/types/EdgeResponse';
import type { NodeResponse } from '$lib/types/NodeResponse';
import { parentPathOf } from './paths';

export const CANVAS_NODE_WIDTH = 190;
export const CANVAS_NODE_HEIGHT = 96;
/** Inner padding of a path box, and the extra room its header takes on top. */
export const PATH_PADDING = 24;
export const PATH_HEADER = 56;
export const PATH_MIN_WIDTH = 260;
export const PATH_MIN_HEIGHT = 170;
const GAP = 80;

export type Point = { x: number; y: number };
export type Size = { width: number; height: number };

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

const cardSize: Size = { width: CANVAS_NODE_WIDTH, height: CANVAS_NODE_HEIGHT };

/**
 * Positions for one set of siblings. Nodes the user has placed keep their
 * saved position; the rest are laid out left-to-right by dagre (among
 * themselves) and parked below everything already placed, so they never land
 * on top of a hand-placed node. `origin` offsets the auto-laid-out block
 * (inside a path, that's below its header).
 */
export function layoutPositions(
	nodes: NodeResponse[],
	edges: EdgeResponse[],
	sizeOf: (node: NodeResponse) => Size = () => cardSize,
	origin: Point = { x: 0, y: 0 }
): Map<string, Point> {
	const positions = new Map<string, Point>();
	const unplaced = nodes.filter((node) => node.canvas_x === null || node.canvas_y === null);
	let placedBottom = -Infinity;
	let placedLeft = Infinity;

	for (const node of nodes) {
		if (node.canvas_x === null || node.canvas_y === null) continue;
		positions.set(node.id, { x: node.canvas_x, y: node.canvas_y });
		placedBottom = Math.max(placedBottom, node.canvas_y + sizeOf(node).height);
		placedLeft = Math.min(placedLeft, node.canvas_x);
	}
	if (!unplaced.length) return positions;

	const layoutGraph = new dagre.graphlib.Graph();
	layoutGraph.setGraph({ rankdir: 'LR', nodesep: 40, ranksep: GAP });
	layoutGraph.setDefaultEdgeLabel(() => ({}));
	const unplacedIds = new Set(unplaced.map((node) => node.id));
	for (const node of unplaced) layoutGraph.setNode(node.id, { ...sizeOf(node) });
	for (const edge of edges) {
		const { source, target } = flowDirection(edge);
		if (unplacedIds.has(source) && unplacedIds.has(target)) layoutGraph.setEdge(source, target);
	}
	dagre.layout(layoutGraph);

	// dagre reports centres; shift the block below the placed nodes, if any.
	const offsetX = Number.isFinite(placedLeft) ? placedLeft : origin.x;
	const offsetY = Number.isFinite(placedBottom) ? placedBottom + GAP : origin.y;
	for (const node of unplaced) {
		const { x, y } = layoutGraph.node(node.id);
		const { width, height } = sizeOf(node);
		positions.set(node.id, { x: x - width / 2 + offsetX, y: y - height / 2 + offsetY });
	}
	return positions;
}

export type CanvasPlacement = {
	/** Relative to the parent path when `parentId` is set, else absolute. */
	position: Point;
	size: Size;
	parentId?: string;
};

/**
 * Lays out the whole canvas, paths included. Each path is a box whose
 * children are positioned relative to it; boxes without a saved size grow
 * to fit their children. Containers are laid out innermost first so a
 * path's size is known before its own siblings are placed.
 */
export function layoutCanvas(
	nodes: NodeResponse[],
	edges: EdgeResponse[]
): Map<string, CanvasPlacement> {
	const shownIds = new Set(nodes.map((node) => node.id));
	const parentOf = new Map(
		[...parentPathOf(edges)].filter(([child, path]) => shownIds.has(child) && shownIds.has(path))
	);
	const placements = new Map<string, CanvasPlacement>();
	const sizes = new Map<string, Size>();
	const sizeOf = (node: NodeResponse) => sizes.get(node.id) ?? cardSize;

	const layoutContainer = (containerId: string | undefined, visiting: Set<string>) => {
		const siblings = nodes.filter((node) => parentOf.get(node.id) === containerId);
		for (const sibling of siblings) {
			if (sibling.kind === 'path' && !visiting.has(sibling.id)) {
				layoutContainer(sibling.id, new Set([...visiting, sibling.id]));
			}
		}
		const origin = containerId ? { x: PATH_PADDING, y: PATH_HEADER } : { x: 0, y: 0 };
		const positions = layoutPositions(siblings, edges, sizeOf, origin);
		for (const sibling of siblings) {
			placements.set(sibling.id, {
				position: positions.get(sibling.id)!,
				size: sizeOf(sibling),
				parentId: containerId
			});
		}

		if (!containerId) return;
		const container = nodes.find((node) => node.id === containerId)!;
		if (container.canvas_width !== null && container.canvas_height !== null) {
			sizes.set(containerId, { width: container.canvas_width, height: container.canvas_height });
			return;
		}
		// Fit the box around its children (plus padding), never below a minimum.
		let right = PATH_MIN_WIDTH;
		let bottom = PATH_MIN_HEIGHT;
		for (const sibling of siblings) {
			const { position, size } = placements.get(sibling.id)!;
			right = Math.max(right, position.x + size.width + PATH_PADDING);
			bottom = Math.max(bottom, position.y + size.height + PATH_PADDING);
		}
		sizes.set(containerId, { width: right, height: bottom });
	};

	layoutContainer(undefined, new Set());
	return placements;
}
