<script lang="ts">
	import { graph } from '$lib/stores/graph.svelte';
	import { boardFilters, clearBoardFilters } from '$lib/stores/filters.svelte';

	type ListKey = 'kinds' | 'statuses' | 'focuses' | 'topicIds';

	const groups = $derived<{ key: ListKey; label: string; options: [string, string][] }[]>([
		{
			key: 'kinds',
			label: 'Kind',
			options: [
				['idea', 'Idea'],
				['project', 'Project'],
				['study', 'Study'],
				['path', 'Path']
			]
		},
		{
			key: 'statuses',
			label: 'Status',
			options: [
				['idea', 'Idea'],
				['queued', 'Queued'],
				['active', 'Active'],
				['paused', 'Paused'],
				['done', 'Done'],
				['archived', 'Archived']
			]
		},
		{
			key: 'focuses',
			label: 'Focus',
			options: [
				['primary', 'Primary'],
				['secondary', 'Secondary'],
				['background', 'Background']
			]
		},
		{ key: 'topicIds', label: 'Tag', options: graph.topics.map((topic) => [topic.id, topic.name]) }
	]);

	function toggle(key: ListKey, value: string) {
		const list = boardFilters[key] as string[];
		boardFilters[key] = (
			list.includes(value) ? list.filter((item) => item !== value) : [...list, value]
		) as never;
	}

	const activeChips = $derived(
		groups.flatMap((group) =>
			(boardFilters[group.key] as string[]).map((value) => ({
				key: group.key,
				value,
				text: `${group.label}: ${group.options.find(([option]) => option === value)?.[1] ?? value}`
			}))
		)
	);
	const anyActive = $derived(
		activeChips.length > 0 ||
			boardFilters.search !== '' ||
			boardFilters.showArchived ||
			boardFilters.showBacklog
	);
</script>

<div class="filter-bar">
	<input
		class="field search"
		type="search"
		placeholder="Filter by title…"
		aria-label="Filter nodes by title"
		bind:value={boardFilters.search}
	/>

	<!-- Native <details> dropdowns: no JS needed for open/close. -->
	{#each groups as group (group.key)}
		<details class="menu">
			<summary>{group.label} ▾</summary>
			<div class="options">
				{#each group.options as [value, label] (value)}
					<label>
						<input
							type="checkbox"
							checked={(boardFilters[group.key] as string[]).includes(value)}
							onchange={() => toggle(group.key, value)}
						/>
						{label}
					</label>
				{:else}
					<span class="muted">None yet</span>
				{/each}
			</div>
		</details>
	{/each}

	<label class="archived">
		<input type="checkbox" bind:checked={boardFilters.showBacklog} /> Show backlog
	</label>

	<label class="archived">
		<input type="checkbox" bind:checked={boardFilters.showArchived} /> Show archived
	</label>

	{#each activeChips as chip (chip.key + chip.value)}
		<button type="button" class="chip" onclick={() => toggle(chip.key, chip.value)}>
			{chip.text} ✕
		</button>
	{/each}

	{#if anyActive}
		<button type="button" class="clear" onclick={clearBoardFilters}>Clear</button>
	{/if}
	<span class="note">filters persist across all three views</span>
</div>

<svelte:window
	onclick={(event) => {
		// Close any open dropdown when clicking outside it.
		for (const menu of document.querySelectorAll('.filter-bar details[open]')) {
			if (!menu.contains(event.target as Node)) menu.removeAttribute('open');
		}
	}}
/>

<style>
	.filter-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	.search {
		width: 200px;
		padding: 7px 10px;
	}

	.menu {
		position: relative;
	}

	summary {
		list-style: none;
		cursor: pointer;
		font: 600 12px/1 var(--font-display);
		color: var(--ink-3);
		background: var(--surface);
		border: var(--border-width-hair) solid var(--line);
		padding: 8px 11px;
		user-select: none;
	}

	summary::-webkit-details-marker {
		display: none;
	}

	.menu[open] summary {
		border-color: var(--ink);
		color: var(--ink);
	}

	.options {
		animation: drop-in var(--fast) var(--ease);
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		z-index: 30;
		min-width: 160px;
		max-height: 280px;
		overflow-y: auto;
		background: var(--surface);
		border: var(--border-width) solid var(--frame);
		padding: 6px;
		display: flex;
		flex-direction: column;
	}

	.options label,
	.archived {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 8px;
		font: 600 12.5px/1 var(--font-display);
		color: var(--ink);
		cursor: pointer;
		white-space: nowrap;
	}

	.options label:hover {
		background: var(--surface-2);
	}

	.archived {
		color: var(--ink-3);
	}

	input[type='checkbox'] {
		accent-color: var(--accent);
		margin: 0;
	}

	.chip {
		font: 600 12px/1 var(--font-display);
		color: var(--ink-3);
		background: var(--surface-2);
		border: var(--border-width-hair) solid var(--line);
		padding: 7px 11px;
	}

	.clear {
		border: none;
		background: none;
		font: 700 12px/1 var(--font-display);
		color: var(--accent);
		padding: 7px 4px;
	}

	.note {
		font: 600 11.5px/1 var(--font-display);
		color: var(--ink-2);
	}
</style>
