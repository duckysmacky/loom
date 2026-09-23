<script lang="ts">
	import { Handle, Position, type NodeProps } from '@xyflow/svelte';
	import Badge from '$lib/components/ui/Badge.svelte';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';
	import { TIER_COLOR, accentColor, borderStyle, progressPair } from '$lib/graph/display';
	import { CANVAS_NODE_WIDTH } from '$lib/graph/layout';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	let { data, selected }: NodeProps & { data: { node: NodeResponse; dimmed: boolean } } = $props();

	const node = $derived(data.node);
	const kind = $derived(node.kind);
	const border = $derived(borderStyle(node));
	const progress = $derived(progressPair(node));
</script>

<!-- Target handle on the left, source on the right: edges run prerequisite
→ dependent and child → container, left to right. -->
<Handle type="target" position={Position.Left} />
<div
	class="canvas-node line-{border.line} tone-{border.tone}"
	class:dimmed={data.dimmed}
	class:selected
	style:width="{CANVAS_NODE_WIDTH}px"
	style:border-top-color={TIER_COLOR[node.focus]}
>
	<div class="head">
		<span class="accent" style:background={accentColor(node)}></span>
		<span class="kind">{kind}</span>
		<Badge status={node.status} blocked={node.blocked} />
	</div>
	<div class="title">{node.title}</div>
	{#if progress}
		<div class="progress">
			<ProgressBar value={progress[0]} total={progress[1]} height={5} />
			<span>{progress[0]}/{progress[1]}</span>
		</div>
	{/if}
	<div
		class="tier"
		style:color={node.focus === 'background' ? 'var(--ink-2)' : TIER_COLOR[node.focus]}
	>
		{node.focus}
	</div>
</div>
<Handle type="source" position={Position.Right} />

<style>
	.canvas-node {
		background: var(--surface);
		border: var(--border-width) solid var(--frame);
		/* The top edge carries the focus tier color. */
		border-top-width: 5px;
		padding: 10px 12px;
		color: var(--ink);
		font-family: var(--font-display);
	}

	.tier {
		margin-top: 8px;
		font: 600 9px/1 var(--font-mono);
		letter-spacing: 0.14em;
		text-transform: uppercase;
	}

	.line-dashed {
		border-style: dashed;
	}

	.line-dotted {
		border-style: dotted;
	}

	.tone-warn {
		border-color: var(--warn);
	}

	.tone-ok {
		border-color: var(--ok);
	}

	.selected {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}

	.dimmed {
		opacity: 0.28;
	}

	.head {
		display: flex;
		align-items: center;
		gap: 7px;
	}

	.head :global(.badge) {
		padding: 3px 6px;
		font-size: 9px;
		margin-left: auto;
	}

	.accent {
		width: 8px;
		height: 8px;
		flex: none;
	}

	.kind {
		font: 700 9px/1 var(--font-mono);
		letter-spacing: 0.12em;
		text-transform: uppercase;
		color: var(--ink-2);
	}

	.title {
		margin-top: 7px;
		font: 700 13px/1.2 var(--font-display);
		overflow-wrap: anywhere;
	}

	.progress {
		margin-top: 8px;
		display: flex;
		align-items: center;
		gap: 8px;
		font: 700 10px/1 var(--font-mono);
		color: var(--ink-3);
	}
</style>
