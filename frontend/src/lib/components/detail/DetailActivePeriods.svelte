<script lang="ts">
	import { slide } from 'svelte/transition';
	import { periodsApi } from '$lib/api/endpoints';
	import { fromDateInput, toDateInput } from '$lib/graph/display';
	import { ms } from '$lib/motion';
	import { graph } from '$lib/stores/graph.svelte';
	import { notifyError } from '$lib/stores/toasts.svelte';
	import type { ActivePeriodResponse } from '$lib/types/ActivePeriodResponse';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	/**
	 * Manual active/paused history, independent of the "Track active
	 * periods" setting - always available, for backfilling or correcting
	 * what the auto-tracking missed (or when it's off entirely).
	 */
	let { node }: { node: NodeResponse } = $props();

	let periods = $state<ActivePeriodResponse[]>([]);

	// Refetch when the node changes or the graph reloads after a mutation.
	$effect(() => {
		void graph.version;
		periodsApi
			.list(node.id)
			.then((loaded) => (periods = loaded))
			.catch(notifyError);
	});

	function addPeriod() {
		graph.mutate(() => periodsApi.add(node.id, { started_at: new Date().toISOString() }));
	}

	function setStarted(period: ActivePeriodResponse, value: string) {
		const started_at = fromDateInput(value);
		// The column isn't nullable - an emptied field is a no-op, not a clear.
		if (started_at) graph.mutate(() => periodsApi.update(period.id, { started_at }));
	}

	function setEnded(period: ActivePeriodResponse, value: string) {
		graph.mutate(() => periodsApi.update(period.id, { ended_at: fromDateInput(value) }));
	}
</script>

<section>
	<div class="head">
		<span class="label">Active periods</span>
		<button type="button" class="link" onclick={addPeriod}>+ Add period</button>
	</div>
	<p class="explain">
		The first period's start is the Started date, the last one's end the Completed date.
	</p>

	{#if periods.length}
		<ul class="items">
			{#each periods as period, index (period.id)}
				<li class="item" transition:slide={{ duration: ms(160) }}>
					{#if periods.length > 1}
						<span class="index">{index + 1}</span>
					{/if}
					<input
						class="field date"
						type="date"
						aria-label="Started"
						value={toDateInput(period.started_at)}
						onchange={(event) => setStarted(period, event.currentTarget.value)}
					/>
					<span class="arrow">→</span>
					<input
						class="field date"
						type="date"
						aria-label="Ended (blank = ongoing)"
						min={toDateInput(period.started_at)}
						value={toDateInput(period.ended_at)}
						onchange={(event) => setEnded(period, event.currentTarget.value)}
					/>
					<button
						type="button"
						class="remove"
						aria-label="Delete this period"
						onclick={() => graph.mutate(() => periodsApi.remove(period.id))}>✕</button
					>
				</li>
			{/each}
		</ul>
	{/if}
</section>

<style>
	.head {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.link {
		margin-left: auto;
		border: none;
		background: none;
		padding: 0;
		font: 700 11px/1 var(--font-display);
		color: var(--accent);
	}

	.explain {
		margin: 4px 0 0;
		font: 500 11.5px/1.4 var(--font-display);
		color: var(--ink-2);
	}

	.items {
		list-style: none;
		margin: 10px 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
	}

	.item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 4px;
		border-bottom: var(--border-width-hair) solid var(--surface-2);
	}

	.item:hover {
		background: var(--surface-2);
	}

	.date {
		padding: 5px 6px;
		font-size: 12px;
	}

	.arrow {
		flex: none;
		color: var(--ink-2);
	}

	.index {
		flex: none;
		width: 14px;
		font: 700 11px/1 var(--font-mono);
		color: var(--ink-2);
		text-align: right;
	}

	.remove {
		margin-left: auto;
		border: none;
		background: none;
		padding: 2px 6px;
		font-size: 11px;
		color: var(--ink-2);
		opacity: 0;
	}

	.item:hover .remove,
	.remove:focus-visible {
		opacity: 1;
	}

	.remove:hover {
		color: var(--warn);
	}
</style>
