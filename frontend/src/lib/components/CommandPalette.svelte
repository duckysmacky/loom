<script lang="ts">
	import { goto } from '$app/navigation';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { accentColor, displayKind } from '$lib/graph/display';
	import { fuzzyScore } from '$lib/fuzzy';
	import { openNode } from '$lib/navigation';
	import { graph } from '$lib/stores/graph.svelte';
	import { logout } from '$lib/stores/session.svelte';
	import { overlays } from '$lib/stores/ui.svelte';

	type Item = {
		id: string;
		label: string;
		hint: string;
		color?: string;
		run: () => void;
	};

	const MAX_RESULTS = 12;

	let query = $state('');
	let activeIndex = $state(0);

	$effect(() => {
		if (overlays.paletteOpen) query = '';
	});

	const commands: Item[] = [
		{ id: 'go-dashboard', label: 'Go to Dashboard', hint: 'page', run: () => goto('/dashboard') },
		{
			id: 'go-organized',
			label: 'Go to Board · Organized',
			hint: 'page',
			run: () => goto('/board/organized')
		},
		{
			id: 'go-canvas',
			label: 'Go to Board · Canvas',
			hint: 'page',
			run: () => goto('/board/canvas')
		},
		{
			id: 'go-timeline',
			label: 'Go to Board · Timeline',
			hint: 'page',
			run: () => goto('/board/timeline')
		},
		{ id: 'go-nodes', label: 'Go to Nodes · Backlog', hint: 'page', run: () => goto('/nodes') },
		{ id: 'go-settings', label: 'Go to Settings', hint: 'page', run: () => goto('/settings') },
		{
			id: 'capture',
			label: 'Quick capture',
			hint: 'N',
			run: () => (overlays.captureOpen = true)
		},
		{ id: 'sign-out', label: 'Sign out', hint: 'account', run: logout }
	];

	const results = $derived.by(() => {
		const nodeItems: Item[] = graph.nodes.map((node) => ({
			id: node.id,
			label: node.title,
			hint: `${displayKind(node)} · ${node.blocked && node.status !== 'done' ? 'blocked' : node.status}`,
			color: accentColor(node),
			run: () => openNode(node.id)
		}));
		return [...nodeItems, ...commands]
			.flatMap((item) => {
				const score = fuzzyScore(query, item.label);
				return score === null ? [] : [{ item, score }];
			})
			.toSorted((left, right) => right.score - left.score)
			.slice(0, MAX_RESULTS)
			.map(({ item }) => item);
	});

	$effect(() => {
		void results;
		activeIndex = 0;
	});

	function choose(item: Item | undefined) {
		if (!item) return;
		overlays.paletteOpen = false;
		item.run();
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			activeIndex = (activeIndex + 1) % Math.max(results.length, 1);
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			activeIndex = (activeIndex - 1 + results.length) % Math.max(results.length, 1);
		} else if (event.key === 'Enter') {
			event.preventDefault();
			choose(results[activeIndex]);
		}
	}
</script>

<Modal
	open={overlays.paletteOpen}
	onclose={() => (overlays.paletteOpen = false)}
	label="Command palette"
	width={560}
>
	<div class="palette">
		<div class="search">
			<span class="lens"></span>
			<!-- svelte-ignore a11y_autofocus -->
			<input
				placeholder="Jump to a node or run a command…"
				aria-label="Search nodes and commands"
				role="combobox"
				aria-expanded="true"
				aria-controls="palette-results"
				aria-activedescendant={results[activeIndex] ? `palette-${activeIndex}` : undefined}
				autofocus
				bind:value={query}
				{onkeydown}
			/>
			<span class="key">esc</span>
		</div>
		<ul class="results" id="palette-results" role="listbox">
			{#each results as item, index (item.id)}
				<li
					id="palette-{index}"
					role="option"
					aria-selected={index === activeIndex}
					class:active={index === activeIndex}
					onmousemove={() => (activeIndex = index)}
					onclick={() => choose(item)}
					onkeydown={() => {}}
				>
					{#if item.color}
						<span class="dot" style:background={item.color}></span>
					{:else}
						<span class="dot command">›</span>
					{/if}
					<span class="text">{item.label}</span>
					<span class="hint">{item.hint}</span>
				</li>
			{:else}
				<li class="empty">No matches</li>
			{/each}
		</ul>
	</div>
</Modal>

<style>
	.search {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 14px 16px;
		border-bottom: var(--border-width) solid var(--frame);
	}

	.lens {
		width: 12px;
		height: 12px;
		border: 1.5px solid var(--ink-2);
		border-radius: 50%;
		flex: none;
	}

	input {
		flex: 1;
		border: none;
		background: none;
		font: 600 15px/1.2 var(--font-display);
		color: var(--ink);
	}

	input:focus {
		outline: none;
	}

	.key {
		font: 600 10.5px/1 var(--font-mono);
		color: var(--ink-2);
		background: var(--chip);
		padding: 4px 6px;
	}

	.results {
		list-style: none;
		margin: 0;
		padding: 6px;
		max-height: 380px;
		overflow-y: auto;
	}

	li {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 10px;
		cursor: pointer;
	}

	li.active {
		background: var(--chip);
	}

	.dot {
		width: 8px;
		height: 8px;
		flex: none;
	}

	.dot.command {
		width: 8px;
		height: auto;
		font: 700 13px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.text {
		flex: 1;
		min-width: 0;
		font: 700 13.5px/1.2 var(--font-display);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.hint {
		font: 500 10px/1 var(--font-mono);
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--ink-2);
	}

	.empty {
		cursor: default;
		color: var(--ink-2);
	}
</style>
