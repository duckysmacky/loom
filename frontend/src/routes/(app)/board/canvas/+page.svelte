<script lang="ts">
	import '@xyflow/svelte/dist/base.css';
	import { Background, SvelteFlow, type Connection } from '@xyflow/svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { edgesApi, nodesApi } from '$lib/api/endpoints';
	import { flowDirection, layoutPositions } from '$lib/graph/layout';
	import { openNode } from '$lib/navigation';
	import { boardFilters, matchesBoardFilters } from '$lib/stores/filters.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify, notifyError } from '$lib/stores/toasts.svelte';
	import type { CreateEdgeRequest } from '$lib/types/CreateEdgeRequest';
	import CanvasControls from './CanvasControls.svelte';
	import CanvasEdge from './CanvasEdge.svelte';
	import CanvasNode from './CanvasNode.svelte';
	import type { LoomFlowEdge, LoomFlowNode } from './types';

	const nodeTypes = { loom: CanvasNode };
	const edgeTypes = { loom: CanvasEdge };

	let nodes = $state.raw<LoomFlowNode[]>([]);
	let edges = $state.raw<LoomFlowEdge[]>([]);
	let zoom = $state(1);
	let pendingConnection = $state<Connection | null>(null);

	// Archived nodes leave the canvas entirely unless the filters ask for
	// them; every other non-matching node stays in place, dimmed, so the
	// graph's shape doesn't jump around while filtering.
	const hidden = (status: string) =>
		status === 'archived' &&
		!boardFilters.showArchived &&
		!boardFilters.statuses.includes('archived');

	// Auto-layout depends on which nodes are still unplaced and where the
	// placed ones sit, so it shifts whenever anything is dragged. Persisting a
	// node's first auto-laid-out position pins it: after that it only moves
	// when the user moves it. Plain Set, deliberately not reactive.
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- bookkeeping only, must not trigger the effect
	const pinning = new Set<string>();

	$effect(() => {
		const shown = graph.nodes.filter((node) => !hidden(node.status));
		const shownIds = new Set(shown.map((node) => node.id));
		const positions = layoutPositions(shown, graph.edges);

		for (const node of shown) {
			if (node.canvas_x !== null || pinning.has(node.id)) continue;
			pinning.add(node.id);
			persistPosition(node.id, positions.get(node.id)!).finally(() => pinning.delete(node.id));
		}
		const dimmedIds = new Set(
			shown.filter((node) => !matchesBoardFilters(node)).map((node) => node.id)
		);

		nodes = shown.map((node) => ({
			id: node.id,
			type: 'loom' as const,
			position: positions.get(node.id)!,
			data: { node, dimmed: dimmedIds.has(node.id) },
			deletable: false
		}));
		edges = graph.edges
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

	async function savePositions({ nodes: dragged }: { nodes: LoomFlowNode[] }) {
		for (const node of dragged) await persistPosition(node.id, node.position);
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
			{
				label: `${title(source)} is part of ${title(target)}`,
				request: { from_node_id: source, to_node_id: target, kind: 'part_of' }
			},
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
		onnodeclick={({ node }) => openNode(node.id)}
		onnodedragstop={savePositions}
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
