<script lang="ts">
	import '@xyflow/svelte/dist/base.css';
	import { Background, ConnectionMode, SvelteFlow } from '@xyflow/svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import CanvasEdge from '$lib/components/canvas/CanvasEdge.svelte';
	import CanvasNode from '$lib/components/canvas/CanvasNode.svelte';
	import CanvasPath from '$lib/components/canvas/CanvasPath.svelte';
	import { toFlowEdges, toFlowNodes } from '$lib/components/canvas/flow';
	import type { LoomFlowEdge, LoomFlowNode } from '$lib/components/canvas/types';
	import { standIns } from '$lib/graph/collapse';
	import {
		CANVAS_NODE_HEIGHT,
		CANVAS_NODE_WIDTH,
		PATH_HEADER,
		grownToFit,
		layoutCanvas,
		splitPlaced
	} from '$lib/graph/layout';
	import { ancestorPaths, parentPathOf } from '$lib/graph/paths';
	import { openNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { isPathCollapsed, prefs } from '$lib/stores/prefs.svelte';
	import { notifyError } from '$lib/stores/toasts.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	/**
	 * The inside of one path as the board canvas draws it: the path itself
	 * isn't drawn, so its children's saved positions (relative to the path)
	 * are used as-is and moving a card here moves it on the board too.
	 * ponytail: move + view only - connecting, and dragging into/out of
	 * paths, stay on the board canvas.
	 */
	let { path }: { path: NodeResponse } = $props();

	const nodeTypes = { loom: CanvasNode, loomPath: CanvasPath };
	const edgeTypes = { loom: CanvasEdge };

	let nodes = $state.raw<LoomFlowNode[]>([]);
	let edges = $state.raw<LoomFlowEdge[]>([]);
	let unplacedCount = $state(0);

	$effect(() => {
		const parentOf = parentPathOf(graph.edges);
		const inside = graph.nodes.filter((node) => ancestorPaths(node.id, parentOf).includes(path.id));
		// Collapsing is shared with the board canvas; only paths in here count.
		const collapsed = new Set(
			inside
				.filter((node) => node.kind === 'path' && isPathCollapsed('canvas', node.id))
				.map((node) => node.id)
		);
		const standIn = standIns(
			inside.map((node) => node.id),
			parentOf,
			collapsed
		);
		const shown = inside.filter((node) => !standIn.has(node.id));
		// Unplaced nodes aren't pinned here; the board canvas does that.
		const split = prefs.autoPlace ? { placed: shown, unplaced: [] } : splitPlaced(shown, parentOf);
		unplacedCount = split.unplaced.length;
		const drawn = split.placed;
		const placements = layoutCanvas(drawn, graph.edges, collapsed);
		const shownIds = new Set(drawn.map((node) => node.id));
		nodes = toFlowNodes(drawn, placements, collapsed, parentOf, new Set()).map((flowNode) =>
			// Cards in a nested path stay inside its box (no reparenting here).
			flowNode.parentId ? { ...flowNode, extent: 'parent' as const } : flowNode
		);
		edges = toFlowEdges(graph.edges, standIn, shownIds, placements, new Set(), graph.nodeById);
	});

	async function settleDrag({ nodes: dragged }: { nodes: LoomFlowNode[] }) {
		const draggedIds = new Set(dragged.map((flowNode) => flowNode.id));
		let box =
			path.canvas_width !== null && path.canvas_height !== null
				? { width: path.canvas_width, height: path.canvas_height }
				: null;
		try {
			for (const moved of dragged) {
				if (moved.parentId && draggedIds.has(moved.parentId)) continue;
				let { x, y } = moved.position;
				// Directly inside the path: keep clear of its top-left edge and header.
				if (!moved.parentId) {
					x = Math.max(0, x);
					y = Math.max(PATH_HEADER, y);
					if (box && prefs.pathAutoExpand) {
						const size = {
							width: moved.width ?? moved.measured?.width ?? CANVAS_NODE_WIDTH,
							height: moved.height ?? moved.measured?.height ?? CANVAS_NODE_HEIGHT
						};
						box = grownToFit(box, { x, y }, size);
					}
				}
				graph.replaceNode(
					await nodesApi.update(moved.id, { canvas_x: Math.round(x), canvas_y: Math.round(y) })
				);
			}
			if (
				box &&
				(Math.round(box.width) !== path.canvas_width ||
					Math.round(box.height) !== path.canvas_height)
			) {
				graph.replaceNode(
					await nodesApi.update(path.id, {
						canvas_width: Math.round(box.width),
						canvas_height: Math.round(box.height)
					})
				);
			}
		} catch (error) {
			notifyError(error);
			await graph.load();
		}
	}
</script>

<div class="path-canvas">
	{#if !nodes.length}
		<div class="empty">
			{unplacedCount ? 'Nothing placed yet.' : 'Nothing inside yet.'}
		</div>
	{/if}
	<SvelteFlow
		bind:nodes
		bind:edges
		{nodeTypes}
		{edgeTypes}
		fitView
		minZoom={0.05}
		maxZoom={2}
		nodesConnectable={false}
		// Every handle is a source (SideHandles); strict mode would drop the edges.
		connectionMode={ConnectionMode.Loose}
		deleteKey={null}
		proOptions={{ hideAttribution: true }}
		onnodeclick={({ node, event }) => {
			// Same rule as the board: a path box opens from its header only.
			const onHeader = (event.target as Element | null)?.closest('.path-head');
			if (node.type !== 'loomPath' || onHeader) openNode(node.id);
		}}
		onnodedragstop={settleDrag}
	>
		<Background patternColor="var(--grid-dot)" bgColor="var(--bg)" gap={26} size={1.4} />
	</SvelteFlow>
</div>
{#if unplacedCount}
	<p class="note">
		{unplacedCount} unplaced - place {unplacedCount === 1 ? 'it' : 'them'} on the board canvas.
	</p>
{/if}

<style>
	.path-canvas {
		position: relative;
		height: 380px;
		margin-top: 10px;
		border: var(--border-width-hair) dashed var(--node-path);
	}

	.path-canvas :global(.svelte-flow) {
		--xy-node-border-radius: 0;
		--xy-handle-background-color: var(--surface);
		--xy-handle-border-color: var(--ink-2);
		--xy-selection-background-color: var(--selection-tint);
		--xy-selection-border: 1px dashed var(--accent);
		font-family: var(--font-display);
	}

	.path-canvas :global(.svelte-flow__handle) {
		width: 9px;
		height: 9px;
		border-radius: 0;
		border-width: 1.5px;
	}

	.empty {
		position: absolute;
		inset: 45% 0 auto;
		z-index: 5;
		text-align: center;
		font: 500 12.5px/1 var(--font-display);
		color: var(--ink-2);
		pointer-events: none;
	}

	.note {
		margin: 6px 0 0;
		font: 500 12px/1.3 var(--font-display);
		color: var(--ink-2);
	}
</style>
