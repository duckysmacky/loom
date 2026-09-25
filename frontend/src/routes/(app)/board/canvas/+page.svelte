<script lang="ts">
	import '@xyflow/svelte/dist/base.css';
	import { Background, SvelteFlow, type Connection } from '@xyflow/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { edgesApi, nodesApi } from '$lib/api/endpoints';
	import { isBacklog } from '$lib/graph/display';
	import {
		CANVAS_NODE_HEIGHT,
		CANVAS_NODE_WIDTH,
		flowDirection,
		layoutCanvas
	} from '$lib/graph/layout';
	import { ancestorPaths, nestingDepth, parentPathOf } from '$lib/graph/paths';
	import { openNode } from '$lib/navigation';
	import { boardFilters, matchesBoardFilters } from '$lib/stores/filters.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify, notifyError } from '$lib/stores/toasts.svelte';
	import type { CreateEdgeRequest } from '$lib/types/CreateEdgeRequest';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import CanvasControls from './CanvasControls.svelte';
	import CanvasEdge from './CanvasEdge.svelte';
	import CanvasNode from './CanvasNode.svelte';
	import CanvasPath from './CanvasPath.svelte';
	import type { LoomFlowEdge, LoomFlowNode } from './types';

	const nodeTypes = { loom: CanvasNode, loomPath: CanvasPath };
	const edgeTypes = { loom: CanvasEdge };

	let nodes = $state.raw<LoomFlowNode[]>([]);
	let edges = $state.raw<LoomFlowEdge[]>([]);
	let zoom = $state(1);
	let pendingConnection = $state<Connection | null>(null);

	// Backlog ideas leave the canvas unless "Show backlog" is on, and archived
	// nodes leave it unless the filters ask for them. Every other
	// non-matching node stays in place, dimmed, so the graph's shape doesn't
	// jump around while filtering.
	const hidden = (node: NodeResponse) =>
		(isBacklog(node) && !boardFilters.showBacklog) ||
		(node.status === 'archived' &&
			!boardFilters.showArchived &&
			!boardFilters.statuses.includes('archived'));

	// Auto-layout depends on which nodes are still unplaced and where the
	// placed ones sit, so it shifts whenever anything is dragged. Persisting a
	// node's first auto-laid-out position pins it: after that it only moves
	// when the user moves it. Plain Set, deliberately not reactive.
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- bookkeeping only, must not trigger the effect
	const pinning = new Set<string>();

	$effect(() => {
		const shown = graph.nodes.filter((node) => !hidden(node));
		const shownIds = new Set(shown.map((node) => node.id));
		const placements = layoutCanvas(shown, graph.edges);

		for (const node of shown) {
			if (node.canvas_x !== null || pinning.has(node.id)) continue;
			pinning.add(node.id);
			persistPosition(node.id, placements.get(node.id)!.position).finally(() =>
				pinning.delete(node.id)
			);
		}
		const dimmedIds = new Set(
			shown.filter((node) => !matchesBoardFilters(node)).map((node) => node.id)
		);

		// xyflow needs a parent listed before its children.
		const parentOf = parentPathOf(graph.edges);
		const parentsFirst = shown.toSorted(
			(left, right) => nestingDepth(left.id, parentOf) - nestingDepth(right.id, parentOf)
		);
		nodes = parentsFirst.map((node) => {
			const { position, size, parentId } = placements.get(node.id)!;
			const isPath = node.kind === 'path';
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
		// part_of is drawn as containment (the box), not as a line.
		edges = graph.edges
			.filter((edge) => edge.kind !== 'part_of')
			.filter((edge) => shownIds.has(edge.from_node_id) && shownIds.has(edge.to_node_id))
			.map((edge) => {
				const { source, target } = flowDirection(edge);
				const unmet =
					edge.kind === 'requires' && graph.nodeById.get(edge.to_node_id)?.status !== 'done';
				return {
					id: edge.id,
					source,
					target,
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

	/**
	 * Drag stop doubles as "drop into / out of a path": the innermost path box
	 * under the node's centre becomes its parent. Same parent → just save the
	 * (relative) position; different parent → move the part_of edge, then save
	 * the position relative to the new parent.
	 */
	async function settleDrag({ nodes: dragged }: { nodes: LoomFlowNode[] }) {
		const byId = new Map(nodes.map((flowNode) => [flowNode.id, flowNode]));
		const absolute = (id: string) => {
			const point = { x: 0, y: 0 };
			for (let current = byId.get(id); current; current = byId.get(current.parentId ?? '')) {
				point.x += current.position.x;
				point.y += current.position.y;
			}
			return point;
		};
		const parentOf = parentPathOf(graph.edges);
		const draggedIds = new Set(dragged.map((flowNode) => flowNode.id));
		let membershipChanged = false;

		for (const moved of dragged) {
			// Moving along with a dragged ancestor: its relative position is unchanged.
			if (ancestorPaths(moved.id, parentOf).some((id) => draggedIds.has(id))) continue;

			const topLeft = absolute(moved.id);
			const size = sizeOf(moved);
			const centre = { x: topLeft.x + size.width / 2, y: topLeft.y + size.height / 2 };
			const target = nodes
				.filter((candidate) => candidate.type === 'loomPath' && candidate.id !== moved.id)
				.filter((candidate) => !ancestorPaths(candidate.id, parentOf).includes(moved.id))
				.filter((candidate) => {
					const origin = absolute(candidate.id);
					const box = sizeOf(candidate);
					return (
						centre.x >= origin.x &&
						centre.x <= origin.x + box.width &&
						centre.y >= origin.y &&
						centre.y <= origin.y + box.height
					);
				})
				.toSorted(
					(left, right) => nestingDepth(right.id, parentOf) - nestingDepth(left.id, parentOf)
				)[0];

			if (target?.id === moved.parentId) {
				await persistPosition(moved.id, moved.position);
				continue;
			}

			membershipChanged = true;
			const previous = graph.edges.find(
				(edge) => edge.kind === 'part_of' && edge.from_node_id === moved.id
			);
			const base = target ? absolute(target.id) : { x: 0, y: 0 };
			try {
				if (previous) await edgesApi.remove(previous.id);
				if (target) {
					await edgesApi.create({ from_node_id: moved.id, to_node_id: target.id, kind: 'part_of' });
				}
				await nodesApi.update(moved.id, {
					canvas_x: Math.round(topLeft.x - base.x),
					canvas_y: Math.round(topLeft.y - base.y)
				});
				const movedTitle = moved.data.node.title;
				notify(
					target
						? `Moved “${movedTitle}” into “${target.data.node.title}”`
						: `Moved “${movedTitle}” out of its path`
				);
			} catch (error) {
				notifyError(error);
			}
		}
		if (membershipChanged) await graph.load();
	}

	const title = (id: string | undefined) => (id ? graph.nodeById.get(id)?.title : '') ?? '';

	// A drag from S's right handle to T's left handle offers the relationships
	// that match the canvas's left-to-right reading.
	const connectionOptions = $derived.by(() => {
		if (!pendingConnection) return [];
		const { source, target } = pendingConnection;
		const options: { label: string; request: CreateEdgeRequest }[] = [
			{
				label: `${title(target)} requires ${title(source)}`,
				request: { from_node_id: target, to_node_id: source, kind: 'requires' }
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

<div class="canvas">
	{#if graph.loaded && !graph.nodes.length}
		<div class="empty">No nodes yet. Press N to capture one.</div>
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
	>
		<Background patternColor="var(--grid-dot)" bgColor="var(--bg)" gap={26} size={1.4} />
		<CanvasControls {zoom} />
	</SvelteFlow>
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
	.canvas {
		flex: 1;
		min-height: 480px;
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
