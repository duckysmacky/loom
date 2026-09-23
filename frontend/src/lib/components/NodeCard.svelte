<script lang="ts">
	import Badge from '$lib/components/ui/Badge.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';
	import { nodesApi } from '$lib/api/endpoints';
	import {
		accentColor,
		borderStyle,
		incrementedProgress,
		notesExcerpt,
		progressPair,
		progressText,
		relativeDays,
		requirementsOf
	} from '$lib/graph/display';
	import { graph } from '$lib/stores/graph.svelte';
	import { prefs } from '$lib/stores/prefs.svelte';
	import { notify } from '$lib/stores/toasts.svelte';
	import { openNode } from '$lib/navigation';
	import type { NodeResponse } from '$lib/types/NodeResponse';
	import type { Snippet } from 'svelte';

	let {
		node,
		feature = false,
		showPoke = prefs.pokeFromCards,
		children
	}: {
		node: NodeResponse;
		/** Dashboard-size card with the notes excerpt and "touched" line. */
		feature?: boolean;
		showPoke?: boolean;
		children?: Snippet;
	} = $props();

	const kind = $derived(node.kind);
	const border = $derived(borderStyle(node));
	const unmet = $derived(
		requirementsOf(node.id, graph.edges, graph.nodeById).filter((requirement) => !requirement.met)
	);
	const progress = $derived(progressPair(node));
	const pokeable = $derived(
		showPoke && (node.status === 'active' || node.status === 'queued' || node.status === 'paused')
	);

	const nextProgress = $derived(incrementedProgress(node));

	function increment(event: MouseEvent) {
		// The card itself opens the detail view - keep the click here.
		event.stopPropagation();
		if (nextProgress !== null) {
			graph.mutate(() => nodesApi.update(node.id, { progress_current: nextProgress }));
		}
	}

	async function poke(event: MouseEvent) {
		event.stopPropagation();
		const poked = await graph.mutate(() => nodesApi.poke(node.id));
		if (poked) notify(`Poked “${node.title}”`);
	}
</script>

<!-- NodeCard: 2px border, squared, no shadow. The border carries state - see
borderStyle(): dotted idea, dashed not-started, warn blocked, green done. -->
<div
	class="card line-{border.line} tone-{border.tone}"
	class:feature
	role="button"
	tabindex="0"
	onclick={() => openNode(node.id)}
	onkeydown={(event) => {
		if (event.target === event.currentTarget && (event.key === 'Enter' || event.key === ' ')) {
			event.preventDefault();
			openNode(node.id);
		}
	}}
>
	<div class="head">
		<span class="accent" style:background={accentColor(node)}></span>
		<span class="kind">{kind}</span>
		<Badge status={node.status} blocked={node.blocked} />
		{#if feature}
			<span class="touched">
				{node.last_poked_at ? `touched ${relativeDays(node.last_poked_at)}` : 'never touched'}
			</span>
		{/if}
	</div>

	<div class="title">{node.title}</div>

	<!-- Paths show their contents instead of a description. -->
	{#if node.notes && node.kind !== 'path'}
		<p class="excerpt">{notesExcerpt(node.notes, 3)}</p>
	{/if}

	{#if unmet.length}
		<div class="requires">⟵ requires {unmet.map(({ node }) => node.title).join(', ')} · unmet</div>
	{:else if progress}
		<div class="progress">
			<ProgressBar value={progress[0]} total={progress[1]} />
			<span>{progressText(node)}</span>
			{#if node.progress_total !== null}
				<button
					type="button"
					class="bump"
					disabled={nextProgress === null}
					title="Add one to the progress"
					onclick={increment}>+1</button
				>
			{/if}
		</div>
	{/if}

	{@render children?.()}

	{#if pokeable}
		<div class="actions">
			<Button variant="poke" onclick={poke} title="Log that you worked on this">Poke</Button>
		</div>
	{/if}
</div>

<style>
	.card {
		background: var(--surface);
		border: var(--border-width) solid var(--frame);
		padding: var(--card-pad);
		cursor: pointer;
		display: flex;
		flex-direction: column;
		min-width: 0;
		/* Fills its grid cell, so cards in a row line up. */
		height: 100%;
	}

	.card:hover {
		border-color: var(--accent);
	}

	.card.line-dashed {
		border-style: dashed;
	}

	.card.line-dotted {
		border-style: dotted;
	}

	.card.tone-warn {
		border-color: var(--warn);
	}

	.card.tone-ok {
		border-color: var(--ok);
	}

	.feature {
		padding: 16px 17px;
	}

	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	.accent {
		width: 9px;
		height: 9px;
		flex: none;
	}

	.kind {
		font: 700 9.5px/1 var(--font-mono);
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: var(--ink-2);
	}

	.touched {
		margin-left: auto;
		font: 600 12px/1 var(--font-display);
		color: var(--ink-2);
	}

	.title {
		margin-top: 9px;
		font: 700 17px/1.15 var(--font-display);
		overflow-wrap: anywhere;
	}

	.feature .title {
		margin-top: 11px;
		font-size: 20px;
	}

	.excerpt {
		margin: 7px 0 0;
		font: 500 12.5px/1.5 var(--font-display);
		color: var(--ink-3);
		max-width: 620px;
		display: -webkit-box;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 3;
		line-clamp: 3;
		overflow: hidden;
	}

	.feature .excerpt {
		font-size: 13.5px;
		line-height: 1.55;
	}

	.requires {
		margin-top: 11px;
		font: 600 11.5px/1.35 var(--font-mono);
		color: var(--warn);
	}

	.progress {
		margin-top: 11px;
		display: flex;
		align-items: center;
		gap: 10px;
		font: 700 11.5px/1 var(--font-display);
		color: var(--ink-3);
	}

	.bump {
		flex: none;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		padding: 4px 7px;
		font: 700 10.5px/1 var(--font-mono);
		color: var(--accent);
	}

	.bump:not(:disabled):hover {
		border-color: var(--accent);
	}

	.actions {
		/* Pinned to the bottom of a stretched card. */
		margin-top: auto;
		padding-top: 12px;
		display: flex;
		justify-content: flex-end;
	}
</style>
