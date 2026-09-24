<script lang="ts">
	import { Handle, NodeResizer, Position, type NodeProps } from '@xyflow/svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import { KIND_GLYPH, TIER_COLOR, borderStyle, progressText } from '$lib/graph/display';
	import { PATH_MIN_HEIGHT, PATH_MIN_WIDTH } from '$lib/graph/layout';
	import { graph } from '$lib/stores/graph.svelte';
	import { notifyError } from '$lib/stores/toasts.svelte';
	import type { LoomFlowNode } from './types';

	let { data, selected }: NodeProps<LoomFlowNode> = $props();

	const node = $derived(data.node);
	const border = $derived(borderStyle(node));

	// Saving the size pins the box; position too, since dragging a top or
	// left handle moves the box's origin.
	async function saveSize(params: { x: number; y: number; width: number; height: number }) {
		try {
			const updated = await nodesApi.update(node.id, {
				canvas_x: Math.round(params.x),
				canvas_y: Math.round(params.y),
				canvas_width: Math.round(params.width),
				canvas_height: Math.round(params.height)
			});
			graph.replaceNode(updated);
		} catch (error) {
			notifyError(error);
		}
	}
</script>

<!-- A path is a box: its nodes are laid out inside it (xyflow sub-flow) and
move with it. The fill is translucent so nested paths read as depth. -->
<NodeResizer
	isVisible={selected}
	minWidth={PATH_MIN_WIDTH}
	minHeight={PATH_MIN_HEIGHT}
	color="var(--node-path)"
	onResizeEnd={(_, params) => saveSize(params)}
/>
<Handle type="target" position={Position.Left} />
<div
	class="path-box line-{border.line} tone-{border.tone}"
	class:dimmed={data.dimmed}
	class:selected
	style:border-top-color={TIER_COLOR[node.focus]}
>
	<div class="head">
		<span class="kind"><span class="glyph">{KIND_GLYPH.path}</span> path</span>
		<Badge status={node.status} blocked={node.blocked} />
		<span class="title">{node.title}</span>
		{#if progressText(node)}<span class="count">{progressText(node)}</span>{/if}
	</div>
</div>
<Handle type="source" position={Position.Right} />

<style>
	.path-box {
		width: 100%;
		height: 100%;
		background: var(--path-fill);
		border: var(--border-width) dashed var(--node-path);
		border-top: 5px solid;
		font-family: var(--font-display);
		color: var(--ink);
	}

	.selected {
		border-color: var(--accent);
	}

	.dimmed {
		opacity: 0.35;
	}

	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
	}

	.head :global(.badge) {
		padding: 3px 6px;
		font-size: 9px;
	}

	.kind {
		font: 700 9.5px/1 var(--font-mono);
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: var(--node-path);
	}

	.title {
		font: 700 15px/1.2 var(--font-display);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.count {
		margin-left: auto;
		font: 600 11px/1 var(--font-mono);
		color: var(--ink-2);
		white-space: nowrap;
	}

	/* Same border language as cards: dotted when paused, status colours. */
	.path-box.line-dotted {
		border-style: dotted;
		border-top-style: solid;
	}

	.path-box.tone-warn {
		border-color: var(--warn);
	}

	.path-box.tone-ok {
		border-color: var(--ok);
	}

	.path-box.tone-muted {
		border-color: var(--ink-2);
	}

	.path-box.tone-faded {
		border-color: var(--line);
		opacity: 0.7;
	}

	.glyph {
		font-size: 13px;
	}

	.kind {
		white-space: nowrap;
	}
</style>
