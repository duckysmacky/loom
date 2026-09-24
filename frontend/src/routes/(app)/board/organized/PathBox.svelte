<script lang="ts">
	import { flip } from 'svelte/animate';
	import Badge from '$lib/components/ui/Badge.svelte';
	import NodeCard from '$lib/components/NodeCard.svelte';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';
	import { KIND_GLYPH, TIER_COLOR, borderStyle } from '$lib/graph/display';
	import { openNode } from '$lib/navigation';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import PathBox from './PathBox.svelte';

	let {
		path,
		childrenOf
	}: {
		path: NodeResponse;
		/** The visible nodes inside a path, in display order. */
		childrenOf: (pathId: string) => NodeResponse[];
	} = $props();

	const inside = $derived(childrenOf(path.id));
	const progress = $derived(path.container_progress);
	const border = $derived(borderStyle(path));
</script>

<!-- A path is a box its nodes sit inside; sub-paths nest as boxes within it.
The translucent fill stacks, so deeper nesting reads darker. -->
<section
	class="path-box line-{border.line} tone-{border.tone}"
	style:border-top-color={TIER_COLOR[path.focus]}
>
	<button type="button" class="head" onclick={() => openNode(path.id)}>
		<span class="kind"><span class="glyph">{KIND_GLYPH.path}</span> path</span>
		<Badge status={path.status} blocked={path.blocked} />
		<span class="title">{path.title}</span>
		{#if progress}
			<span class="progress">
				<ProgressBar value={progress.done} total={progress.total} />
				<span>{progress.done} / {progress.total} done</span>
			</span>
		{/if}
	</button>

	{#if inside.length}
		<div class="inside">
			{#each inside as node (node.id)}
				<div
					class={node.kind === 'path' ? 'path-slot' : 'card-slot'}
					animate:flip={{ duration: 200 }}
				>
					{#if node.kind === 'path'}
						<PathBox path={node} {childrenOf} />
					{:else}
						<NodeCard {node} />
					{/if}
				</div>
			{/each}
		</div>
	{:else}
		<p class="empty">
			Empty path. Add nodes from its detail view, or drag them into this box on the canvas.
		</p>
	{/if}
</section>

<style>
	/* Only as wide as its contents (cards wrap once it reaches the page edge). */
	.path-box {
		width: fit-content;
		max-width: 100%;
		min-width: var(--card-w);
		background: var(--path-fill);
		border: var(--border-width) dashed var(--node-path);
		border-top: 5px solid;
		padding: 0 14px 14px;
	}

	.head {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		min-width: 0;
		padding: 12px 0;
		border: none;
		background: none;
		text-align: left;
		color: var(--ink);
	}

	.head:hover .title {
		color: var(--accent);
	}

	.kind {
		font: 700 9.5px/1 var(--font-mono);
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: var(--node-path);
	}

	.title {
		font: 700 17px/1.2 var(--font-display);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.progress {
		margin-left: auto;
		display: flex;
		align-items: center;
		gap: 10px;
		flex: 0 1 180px;
		min-width: 110px;
		font: 600 11.5px/1 var(--font-mono);
		color: var(--ink-2);
		white-space: nowrap;
	}

	.inside {
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
		align-items: flex-start;
	}

	/* Fixed card footprint shared with the Organized page (--card-w/--card-h). */
	.card-slot {
		width: var(--card-w);
		height: var(--card-h);
	}

	.path-slot {
		max-width: 100%;
	}

	.empty {
		margin: 0;
		width: var(--card-w);
		padding: 14px;
		border: var(--border-width-hair) dashed var(--node-path);
		font: 500 12.5px/1.4 var(--font-display);
		color: var(--ink-2);
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
