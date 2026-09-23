<script lang="ts">
	import { accentColor, shortDate } from '$lib/graph/display';
	import { barSpan, pokeOffset, timelineWindow } from '$lib/graph/timeline';
	import { openNode } from '$lib/navigation';
	import { matchesBoardFilters } from '$lib/stores/filters.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	const visible = $derived(graph.nodes.filter(matchesBoardFilters));
	const started = $derived(
		visible
			.filter((node) => node.started_at)
			.toSorted((left, right) => left.started_at!.localeCompare(right.started_at!))
	);
	const notStarted = $derived(visible.filter((node) => !node.started_at));
	const timeWindow = $derived(timelineWindow(started));

	function barTone(node: NodeResponse): string {
		if (node.blocked && node.status !== 'done') return 'blocked';
		return node.status;
	}

	function barCaption(node: NodeResponse): string {
		const from = shortDate(node.started_at!);
		return node.completed_at ? `${from} – ${shortDate(node.completed_at)}` : `since ${from}`;
	}
</script>

<div class="page">
	<div class="sheet" style:--week-count={timeWindow.weeks.length}>
		<div class="row head">
			<div class="name label">Node</div>
			<div class="track weeks">
				{#each timeWindow.weeks as week (week.start)}
					<div class="week">{week.label}</div>
				{/each}
			</div>
		</div>

		{#each started as node (node.id)}
			{@const span = barSpan(node, timeWindow)!}
			{@const poke = pokeOffset(node, timeWindow)}
			<div class="row">
				<button type="button" class="name" onclick={() => openNode(node.id)}>
					<span class="dot" style:background={accentColor(node)}></span>
					<span>
						<span class="title">{node.title}</span>
						<span class="meta">{node.focus} · {barCaption(node)}</span>
					</span>
				</button>
				<div class="track">
					{#each timeWindow.weeks as week (week.start)}
						<div class="week-line"></div>
					{/each}
					<button
						type="button"
						class="bar {barTone(node)}"
						style:left="{span.left}%"
						style:width="{span.width}%"
						title="{node.title} · {node.status} · {barCaption(node)}"
						onclick={() => openNode(node.id)}
					>
						<span>{barTone(node)}</span>
					</button>
					{#if poke !== null}
						<span class="poke" style:left="{poke}%" title="Last poked"></span>
					{/if}
				</div>
			</div>
		{:else}
			<div class="empty">
				Nothing started yet{visible.length ? ' among the filtered nodes' : ''}. A bar appears once a
				node goes active.
			</div>
		{/each}

		<div class="legend">
			<span><span class="swatch active"></span>active</span>
			<span><span class="swatch blocked"></span>blocked — waiting on a requirement</span>
			<span><span class="swatch paused"></span>paused / queued</span>
			<span><span class="swatch done"></span>done</span>
			<span><span class="tick"></span>last poke</span>
			<span class="note">actual history: started → completed, or today</span>
		</div>
	</div>

	{#if notStarted.length}
		<section class="not-started">
			<div class="section-head">
				<span class="bar-mark"></span>
				<h2 class="title-head">Not started</h2>
				<span class="meta-head">{notStarted.length} without a start date</span>
			</div>
			<div class="chips">
				{#each notStarted as node (node.id)}
					<button type="button" class="chip" onclick={() => openNode(node.id)}>
						<span class="dot" style:background={accentColor(node)}></span>
						{node.title}
						<span class="status">{node.status}</span>
					</button>
				{/each}
			</div>
		</section>
	{/if}
</div>

<style>
	.page {
		padding: var(--page-pad);
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.sheet {
		background: var(--surface);
		border: var(--border-width) solid var(--line);
		overflow-x: auto;
	}

	.row {
		display: flex;
		align-items: stretch;
		border-bottom: var(--border-width-hair) solid var(--line);
		min-width: calc(220px + var(--week-count, 4) * 64px);
	}

	.head {
		background: var(--surface-2);
	}

	.name {
		width: 220px;
		flex: none;
		display: flex;
		align-items: center;
		gap: 9px;
		padding: 11px 16px;
		border: none;
		background: none;
		text-align: left;
		color: var(--ink);
	}

	.head .name {
		padding: 10px 16px;
	}

	.dot {
		width: 7px;
		height: 7px;
		flex: none;
	}

	.title {
		display: block;
		font: 700 13px/1.2 var(--font-display);
	}

	.name:hover .title {
		color: var(--accent);
	}

	.meta {
		display: block;
		margin-top: 4px;
		font: 400 9.5px/1.2 var(--font-mono);
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--ink-2);
	}

	.track {
		flex: 1;
		position: relative;
		display: flex;
		min-height: 48px;
	}

	.weeks {
		min-height: 0;
	}

	.week {
		flex: 1;
		min-width: 64px;
		padding: 10px 6px;
		border-left: var(--border-width-hair) solid var(--line);
		text-align: center;
		font: 400 10px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.week-line {
		flex: 1;
		min-width: 64px;
		border-left: var(--border-width-hair) solid var(--line);
	}

	.bar {
		position: absolute;
		top: 17px;
		height: 14px;
		border: none;
		padding: 0 6px;
		overflow: hidden;
		text-align: left;
		font: 700 9.5px/14px var(--font-mono);
		white-space: nowrap;
		background: var(--ok);
		color: var(--on-ok);
	}

	.bar.done {
		background: var(--ink);
		color: var(--on-ink);
	}

	.bar.blocked {
		background: repeating-linear-gradient(135deg, var(--hazard) 0 6px, var(--warn-tint) 6px 12px);
		color: var(--ink);
	}

	.bar.paused,
	.bar.queued,
	.bar.idea,
	.bar.archived {
		background: var(--surface);
		border: 1.5px dashed var(--ink-2);
		color: var(--ink-2);
		line-height: 11px;
	}

	.poke {
		position: absolute;
		top: 12px;
		width: 2px;
		height: 24px;
		background: var(--accent);
		pointer-events: none;
	}

	.empty {
		padding: 28px 18px;
		text-align: center;
		color: var(--ink-2);
		border-bottom: var(--border-width-hair) solid var(--line);
	}

	.legend {
		display: flex;
		align-items: center;
		gap: 14px;
		flex-wrap: wrap;
		padding: 11px 16px;
		font: 600 11.5px/1 var(--font-display);
		color: var(--ink-2);
	}

	.legend > span {
		display: flex;
		align-items: center;
		gap: 7px;
	}

	.swatch {
		width: 18px;
		height: 9px;
		display: inline-block;
		background: var(--ok);
	}

	.swatch.blocked {
		background: var(--hazard);
	}

	.swatch.paused {
		background: none;
		border: 1.5px dashed var(--ink-2);
	}

	.swatch.done {
		background: var(--ink);
	}

	.tick {
		width: 2px;
		height: 12px;
		background: var(--accent);
		display: inline-block;
	}

	.note {
		margin-left: auto;
	}

	.not-started .bar-mark {
		width: 4px;
		height: 15px;
		background: var(--line);
	}

	.section-head {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.title-head {
		margin: 0;
		font: 700 13px/1 var(--font-display);
		letter-spacing: 0.07em;
		text-transform: uppercase;
	}

	.meta-head {
		font: 600 12px/1 var(--font-display);
		color: var(--ink-2);
	}

	.chips {
		margin-top: 12px;
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 8px 11px;
		background: var(--surface);
		border: var(--border-width-hair) dashed var(--line);
		font: 700 12.5px/1 var(--font-display);
		color: var(--ink);
	}

	.chip:hover {
		border-color: var(--ink);
	}

	.status {
		font: 400 9.5px/1 var(--font-mono);
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-2);
	}
</style>
