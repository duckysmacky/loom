<script lang="ts">
	import '@xyflow/svelte/dist/base.css';
	import { Background, SvelteFlow, useSvelteFlow, type Connection } from '@xyflow/svelte';
	import { page } from '$app/state';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { edgesApi, nodesApi } from '$lib/api/endpoints';
	import { isIdea } from '$lib/graph/display';
	import {
		CANVAS_NODE_HEIGHT,
		CANVAS_NODE_WIDTH,
		PATH_HEADER,
		PATH_PADDING,
		flowDirection,
		layoutCanvas,
		splitPlaced,
		type Point
	} from '$lib/graph/layout';
	import { rerouteEdges, standIns } from '$lib/graph/collapse';
	import { ancestorPaths, nestingDepth, parentPathOf } from '$lib/graph/paths';
	import { openNode } from '$lib/navigation';
	import { boardFilters, matchesBoardFilters } from '$lib/stores/filters.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import { isPathCollapsed, prefs } from '$lib/stores/prefs.svelte';
	import { notify, notifyError } from '$lib/stores/toasts.svelte';
	import type { CreateEdgeRequest } from '$lib/types/CreateEdgeRequest';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import CanvasControls from './CanvasControls.svelte';
	import CanvasEdge from './CanvasEdge.svelte';
	import CanvasNode from './CanvasNode.svelte';
	import CanvasPath from './CanvasPath.svelte';
	import type { LoomFlowEdge, LoomFlowNode } from './types';
	import UnplacedPanel, { UNPLACED_DRAG_TYPE } from './UnplacedPanel.svelte';

	const nodeTypes = { loom: CanvasNode, loomPath: CanvasPath };
	const edgeTypes = { loom: CanvasEdge };

	let nodes = $state.raw<LoomFlowNode[]>([]);
	let edges = $state.raw<LoomFlowEdge[]>([]);
	let zoom = $state(1);
	let pendingConnection = $state<Connection | null>(null);
	/** Nodes with no position yet: listed in the side panel, not drawn. */
	let unplaced = $state.raw<NodeResponse[]>([]);
	let canvasElement = $state<HTMLDivElement>();
	const flow = useSvelteFlow();

	// Ideas leave the canvas unless "Show ideas" is on, and archived
	// nodes leave it unless the filters ask for them. Every other
	// non-matching node stays in place, dimmed, so the graph's shape doesn't
	// jump around while filtering.
	const hidden = (node: NodeResponse) =>
		(isIdea(node) && !boardFilters.showIdeas) ||
		(node.status === 'archived' &&
			!boardFilters.showArchived &&
			!boardFilters.statuses.includes('archived'));

	// With auto-place on, auto-layout depends on which nodes are still
	// unplaced and where the placed ones sit, so it shifts whenever anything
	// is dragged. Persisting a node's first auto-laid-out position pins it:
	// after that it only moves when the user moves it. Plain Set,
	// deliberately not reactive.
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- bookkeeping only, must not trigger the effect
	const pinning = new Set<string>();

	$effect(() => {
		const parentOf = parentPathOf(graph.edges);
		// A collapsed path is a single card: what's inside it leaves the
		// canvas and its connections are drawn to the card instead.
		const collapsed = new Set(
			graph.nodes
				.filter((node) => node.kind === 'path' && isPathCollapsed('canvas', node.id))
				.map((node) => node.id)
		);
		const standIn = standIns(
			graph.nodes.map((node) => node.id),
			parentOf,
			collapsed
		);
		const shown = graph.nodes.filter((node) => !hidden(node) && !standIn.has(node.id));
		// By default only placed nodes are drawn; the rest wait in the side
		// panel. With auto-place on, everything is drawn and unplaced nodes get
		// laid out and pinned.
		const split = prefs.autoPlace ? { placed: shown, unplaced: [] } : splitPlaced(shown, parentOf);
		unplaced = split.unplaced;
		const drawn = split.placed;
		const shownIds = new Set(drawn.map((node) => node.id));
		const placements = layoutCanvas(drawn, graph.edges, collapsed);

		for (const node of drawn) {
			if (node.canvas_x !== null || pinning.has(node.id)) continue;
			pinning.add(node.id);
			persistPosition(node.id, placements.get(node.id)!.position).finally(() =>
				pinning.delete(node.id)
			);
		}
		const dimmedIds = new Set(
			drawn.filter((node) => !matchesBoardFilters(node)).map((node) => node.id)
		);

		// xyflow needs a parent listed before its children.
		const parentsFirst = drawn.toSorted(
			(left, right) => nestingDepth(left.id, parentOf) - nestingDepth(right.id, parentOf)
		);
		nodes = parentsFirst.map((node) => {
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
				deletable: false
			};
		});
		// Placements are relative to the parent path; walk up for the canvas y.
		const absoluteY = (id: string): number => {
			const placement = placements.get(id);
			if (!placement) return 0;
			return placement.position.y + (placement.parentId ? absoluteY(placement.parentId) : 0);
		};
		// part_of is drawn as containment (the box), not as a line.
		edges = rerouteEdges(graph.edges, standIn)
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
						underlying.kind === 'requires' &&
						graph.nodeById.get(underlying.to_node_id)?.status !== 'done'
				);
				return {
					id: edge.id,
					source,
					target,
					// A line standing in for hidden connections isn't one edge to delete.
					deletable: !rerouted,
					...(related ? { sourceHandle: 'bottom', targetHandle: 'top' } : {}),
					type: 'loom' as const,
					zIndex: 1,
					data: {
						kind: edge.kind,
						unmet,
						dimmed: dimmedIds.has(edge.from_node_id) || dimmedIds.has(edge.to_node_id)
					}
				};
			});
	});

	async function persistPosition(nodeId: string, position: { x: number; y: number }) {
		try {
			const updated = await nodesApi.update(nodeId, {
				canvas_x: Math.round(position.x),
				canvas_y: Math.round(position.y)
			});
			// Position changes no derived state - patch the cache in place
			// instead of refetching the whole graph.
			graph.replaceNode(updated);
		} catch (error) {
			notifyError(error);
		}
	}

	const sizeOf = (flowNode: LoomFlowNode) => ({
		width: flowNode.width ?? flowNode.measured?.width ?? CANVAS_NODE_WIDTH,
		height: flowNode.height ?? flowNode.measured?.height ?? CANVAS_NODE_HEIGHT
	});

	const flowNodeById = $derived(new Map(nodes.map((flowNode) => [flowNode.id, flowNode])));

	/** A drawn node's canvas position (flow positions are relative to the parent path). */
	function absolute(id: string): Point {
		const point = { x: 0, y: 0 };
		for (
			let current = flowNodeById.get(id);
			current;
			current = flowNodeById.get(current.parentId ?? '')
		) {
			point.x += current.position.x;
			point.y += current.position.y;
		}
		return point;
	}

	/**
	 * The innermost path under `point` that `nodeId` could join: expanded
	 * path boxes, and collapsed paths' cards (dropping onto one joins it).
	 */
	function pathAt(point: Point, nodeId: string): LoomFlowNode | undefined {
		const parentOf = parentPathOf(graph.edges);
		return nodes
			.filter((candidate) => candidate.data.node.kind === 'path' && candidate.id !== nodeId)
			.filter((candidate) => !ancestorPaths(candidate.id, parentOf).includes(nodeId))
			.filter((candidate) => {
				const origin = absolute(candidate.id);
				const box = sizeOf(candidate);
				return (
					point.x >= origin.x &&
					point.x <= origin.x + box.width &&
					point.y >= origin.y &&
					point.y <= origin.y + box.height
				);
			})
			.toSorted(
				(left, right) => nestingDepth(right.id, parentOf) - nestingDepth(left.id, parentOf)
			)[0];
	}

	/**
	 * Moves a node into `target` (or out to the top level) and puts it at the
	 * canvas position `topLeft`. The part_of swap resets its position
	 * server-side, so the position is saved after it.
	 */
	async function changePath(
		nodeId: string,
		target: LoomFlowNode | undefined,
		topLeft: Point
	): Promise<boolean> {
		const previous = graph.edges.find(
			(edge) => edge.kind === 'part_of' && edge.from_node_id === nodeId
		);
		const base = target ? absolute(target.id) : { x: 0, y: 0 };
		try {
			if (previous) await edgesApi.remove(previous.id);
			if (target) {
				await edgesApi.create({ from_node_id: nodeId, to_node_id: target.id, kind: 'part_of' });
			}
			// Inside a collapsed path there's nowhere to put it yet: it stays
			// unplaced until the path expands (auto-place lays it out).
			if (target?.type !== 'loom') {
				await nodesApi.update(nodeId, {
					canvas_x: Math.round(topLeft.x - base.x),
					canvas_y: Math.round(topLeft.y - base.y)
				});
			}
			return true;
		} catch (error) {
			notifyError(error);
			return false;
		}
	}

	/**
	 * Drag stop doubles as "drop into / out of a path": the innermost path box
	 * under the node's centre becomes its parent. Same parent → just save the
	 * (relative) position; different parent → move the part_of edge, then save
	 * the position relative to the new parent.
	 */
	async function settleDrag({ nodes: dragged }: { nodes: LoomFlowNode[] }) {
		const parentOf = parentPathOf(graph.edges);
		const draggedIds = new Set(dragged.map((flowNode) => flowNode.id));
		let membershipChanged = false;

		for (const moved of dragged) {
			// Moving along with a dragged ancestor: its relative position is unchanged.
			if (ancestorPaths(moved.id, parentOf).some((id) => draggedIds.has(id))) continue;

			const topLeft = absolute(moved.id);
			const size = sizeOf(moved);
			const target = pathAt(
				{ x: topLeft.x + size.width / 2, y: topLeft.y + size.height / 2 },
				moved.id
			);

			if (target?.id === moved.parentId) {
				await persistPosition(moved.id, moved.position);
				continue;
			}

			membershipChanged = true;
			if (await changePath(moved.id, target, topLeft)) {
				const movedTitle = moved.data.node.title;
				notify(
					target
						? `Moved “${movedTitle}” into “${target.data.node.title}”`
						: `Moved “${movedTitle}” out of its path`
				);
			}
		}
		if (membershipChanged) await graph.load();
	}

	/**
	 * Places a node from the side panel. Dropped at `pointer`, where it lands
	 * decides its path, same as dragging on the canvas. Without a pointer
	 * (the Place button) it keeps its path: the top of its path's box, or the
	 * middle of the view at the top level.
	 */
	async function placeNode(nodeId: string, pointer?: Point) {
		const parentId = parentPathOf(graph.edges).get(nodeId);
		if (!pointer) {
			if (parentId) {
				await persistPosition(nodeId, { x: PATH_PADDING, y: PATH_HEADER });
				return;
			}
			const rect = canvasElement!.getBoundingClientRect();
			const centre = flow.screenToFlowPosition({
				x: rect.left + rect.width / 2,
				y: rect.top + rect.height / 2
			});
			await persistPosition(nodeId, {
				x: centre.x - CANVAS_NODE_WIDTH / 2,
				y: centre.y - CANVAS_NODE_HEIGHT / 2
			});
			return;
		}
		const topLeft = { x: pointer.x - CANVAS_NODE_WIDTH / 2, y: pointer.y - CANVAS_NODE_HEIGHT / 2 };
		const target = pathAt(pointer, nodeId);
		if (target?.id === parentId) {
			const base = target ? absolute(target.id) : { x: 0, y: 0 };
			await persistPosition(nodeId, { x: topLeft.x - base.x, y: topLeft.y - base.y });
		} else if (await changePath(nodeId, target, topLeft)) {
			await graph.load();
		}
	}

	const dragsNode = (event: DragEvent) => !!event.dataTransfer?.types.includes(UNPLACED_DRAG_TYPE);

	// "Open in canvas" on an unplaced node points at it in the panel.
	const focusId = $derived(page.url.searchParams.get('focus'));
	$effect(() => {
		if (focusId && unplaced.some((node) => node.id === focusId)) prefs.unplacedPanelOpen = true;
	});

	const title = (id: string | undefined) => (id ? graph.nodeById.get(id)?.title : '') ?? '';

	// A drag from S's source handle (right or bottom) to T's target handle
	// (left or top) offers the relationships that match the canvas's
	// left-to-right reading.
	const connectionOptions = $derived.by(() => {
		if (!pendingConnection) return [];
		const { source, target } = pendingConnection;
		const options: { label: string; request: CreateEdgeRequest }[] = [
			{
				label: `${title(target)} requires ${title(source)}`,
				request: { from_node_id: target, to_node_id: source, kind: 'requires' }
			},
			{
				label: `${title(source)} comes before ${title(target)}`,
				request: { from_node_id: source, to_node_id: target, kind: 'precedes' }
			},
			...(graph.nodeById.get(target)?.kind === 'path'
				? [
						{
							label: `${title(source)} goes inside path ${title(target)}`,
							request: { from_node_id: source, to_node_id: target, kind: 'part_of' as const }
						}
					]
				: []),
			{
				label: `${title(source)} is related to ${title(target)}`,
				request: { from_node_id: source, to_node_id: target, kind: 'related' }
			}
		];
		return options;
	});

	async function createEdge(request: CreateEdgeRequest) {
		pendingConnection = null;
		const created = await graph.mutate(() => edgesApi.create(request));
		if (created) notify('Connection added');
	}

	async function deleteEdges({ edges: removed }: { edges: LoomFlowEdge[] }) {
		for (const edge of removed) await graph.mutate(() => edgesApi.remove(edge.id));
		if (removed.length) notify(removed.length === 1 ? 'Connection removed' : 'Connections removed');
	}
</script>

<div class="canvas-row">
	<div class="canvas" bind:this={canvasElement}>
		{#if graph.loaded && !graph.nodes.length}
			<div class="empty">No nodes yet. Press N to capture one.</div>
		{:else if graph.loaded && !nodes.length && unplaced.length}
			<div class="empty">Drag nodes from Unplaced onto the canvas.</div>
		{/if}
		<SvelteFlow
			bind:nodes
			bind:edges
			{nodeTypes}
			{edgeTypes}
			fitView
			minZoom={0.2}
			maxZoom={2}
			deleteKey={['Delete', 'Backspace']}
			proOptions={{ hideAttribution: true }}
			onnodeclick={({ node, event }) => {
				// A path box opens only from its header; clicking the body just
				// selects it (for resizing) without popping the detail view.
				const onHeader = (event.target as Element | null)?.closest('.path-head');
				if (node.type !== 'loomPath' || onHeader) openNode(node.id);
			}}
			onnodedragstop={settleDrag}
			onbeforeconnect={(connection) => {
				pendingConnection = connection;
				return false;
			}}
			onbeforedelete={async ({ edges: doomed }) => ({ nodes: [], edges: doomed })}
			ondelete={deleteEdges}
			onmove={(_, viewport) => (zoom = viewport.zoom)}
			ondragover={(event: DragEvent) => {
				if (!dragsNode(event)) return;
				event.preventDefault();
				event.dataTransfer!.dropEffect = 'move';
			}}
			ondrop={(event: DragEvent) => {
				if (!dragsNode(event)) return;
				event.preventDefault();
				placeNode(
					event.dataTransfer!.getData(UNPLACED_DRAG_TYPE),
					flow.screenToFlowPosition({ x: event.clientX, y: event.clientY })
				);
			}}
		>
			<Background patternColor="var(--grid-dot)" bgColor="var(--bg)" gap={26} size={1.4} />
			<CanvasControls {zoom} />
		</SvelteFlow>
	</div>
	{#if !prefs.autoPlace}
		<UnplacedPanel nodes={unplaced} highlight={focusId} onplace={(nodeId) => placeNode(nodeId)} />
	{/if}
</div>

<Modal
	open={pendingConnection !== null}
	onclose={() => (pendingConnection = null)}
	label="Add connection"
	width={440}
>
	<div class="picker">
		<div class="label">Add connection</div>
		{#each connectionOptions as option (option.request.kind)}
			<button type="button" class="option" onclick={() => createEdge(option.request)}>
				<span class="kind">{option.request.kind.replace('_', ' ')}</span>
				{option.label}
			</button>
		{/each}
		<div class="actions">
			<Button variant="quiet" onclick={() => (pendingConnection = null)}>Cancel</Button>
		</div>
	</div>
</Modal>

<style>
	.canvas-row {
		flex: 1;
		display: flex;
		min-height: 480px;
	}

	.canvas {
		flex: 1;
		min-width: 0;
		position: relative;
	}

	.canvas :global(.svelte-flow) {
		--xy-node-border-radius: 0;
		--xy-handle-background-color: var(--surface);
		--xy-handle-border-color: var(--ink-2);
		--xy-selection-background-color: var(--selection-tint);
		--xy-selection-border: 1px dashed var(--accent);
		--xy-connectionline-stroke: var(--accent);
		--xy-connectionline-stroke-width: 2;
		font-family: var(--font-display);
	}

	.canvas :global(.svelte-flow__handle) {
		width: 9px;
		height: 9px;
		border-radius: 0;
		border-width: 1.5px;
	}

	.canvas :global(.svelte-flow__node:focus-visible) {
		outline: 2px solid var(--accent);
	}

	.empty {
		position: absolute;
		inset: 40% 0 auto;
		z-index: 5;
		text-align: center;
		color: var(--ink-2);
		pointer-events: none;
	}

	.picker {
		padding: 20px 22px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.picker .label {
		margin-bottom: 6px;
	}

	.option {
		display: flex;
		flex-direction: column;
		gap: 5px;
		text-align: left;
		padding: 11px 13px;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		font: 700 13.5px/1.3 var(--font-display);
		color: var(--ink);
	}

	.option:hover {
		border-color: var(--ink);
	}

	.kind {
		font: 700 9.5px/1 var(--font-mono);
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--ink-2);
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		margin-top: 6px;
	}
</style>
