<script lang="ts">
	import { slide } from 'svelte/transition';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';
	import { ms } from '$lib/motion';
	import { checklistApi } from '$lib/api/endpoints';
	import { graph } from '$lib/stores/graph.svelte';
	import { notifyError } from '$lib/stores/toasts.svelte';
	import type { ChecklistItemResponse } from '$lib/types/ChecklistItemResponse';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	let { node }: { node: NodeResponse } = $props();

	let items = $state<ChecklistItemResponse[]>([]);
	let newTitle = $state('');
	let editingId = $state<string | null>(null);
	let editDraft = $state('');

	// Refetch when the node changes or the graph reloads after a mutation.
	$effect(() => {
		void graph.version;
		checklistApi
			.list(node.id)
			.then((loaded) => (items = loaded))
			.catch(notifyError);
	});

	const doneCount = $derived(items.filter((item) => item.done).length);

	async function add(event: SubmitEvent) {
		event.preventDefault();
		const title = newTitle.trim();
		if (!title) return;
		// Clear right away so the next item can be typed while this one saves.
		newTitle = '';
		await graph.mutate(() => checklistApi.add(node.id, title));
	}

	async function rename(item: ChecklistItemResponse) {
		const title = editDraft.trim();
		editingId = null;
		if (title && title !== item.title) {
			await graph.mutate(() => checklistApi.update(item.id, { title }));
		}
	}
</script>

<section>
	<div class="head">
		<span class="label">Checklist</span>
		{#if items.length}
			<span class="count">{doneCount} of {items.length} tasks</span>
		{/if}
	</div>

	{#if items.length}
		<div class="bar"><ProgressBar value={doneCount} total={items.length} height={7} /></div>
	{/if}

	<ul class="items">
		{#each items as item (item.id)}
			<li class="item" class:done={item.done} transition:slide={{ duration: ms(160) }}>
				<input
					type="checkbox"
					checked={item.done}
					aria-label="Mark “{item.title}” {item.done ? 'not done' : 'done'}"
					onchange={() => graph.mutate(() => checklistApi.update(item.id, { done: !item.done }))}
				/>
				{#if editingId === item.id}
					<!-- svelte-ignore a11y_autofocus -->
					<input
						class="field edit"
						aria-label="Task title"
						autofocus
						bind:value={editDraft}
						onblur={() => rename(item)}
						onkeydown={(event) => {
							if (event.key === 'Enter') event.currentTarget.blur();
							if (event.key === 'Escape') {
								event.stopPropagation();
								editingId = null;
							}
						}}
					/>
				{:else}
					<button
						type="button"
						class="title"
						onclick={() => {
							editingId = item.id;
							editDraft = item.title;
						}}>{item.title}</button
					>
				{/if}
				<button
					type="button"
					class="remove"
					aria-label="Delete “{item.title}”"
					onclick={() => graph.mutate(() => checklistApi.remove(item.id))}>✕</button
				>
			</li>
		{/each}
	</ul>

	<form class="add" onsubmit={add}>
		<input class="field" placeholder="Add a task…" aria-label="New task" bind:value={newTitle} />
	</form>
</section>

<style>
	.head {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.count {
		margin-left: auto;
		font: 600 12px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.bar {
		margin-top: 10px;
		display: flex;
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
		gap: 10px;
		padding: 6px 4px;
		border-bottom: var(--border-width-hair) solid var(--surface-2);
	}

	.item:hover {
		background: var(--surface-2);
	}

	input[type='checkbox'] {
		width: 16px;
		height: 16px;
		margin: 0;
		accent-color: var(--ok);
		flex: none;
	}

	.title {
		flex: 1;
		min-width: 0;
		border: none;
		background: none;
		padding: 2px 0;
		text-align: left;
		font: 500 14px/1.35 var(--font-display);
		color: var(--ink);
		overflow-wrap: anywhere;
	}

	.title {
		transition: color var(--normal) var(--ease);
	}

	.done .title {
		text-decoration: line-through;
		color: var(--ink-2);
	}

	.edit {
		flex: 1;
		padding: 5px 8px;
	}

	.remove {
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

	.add {
		margin-top: 10px;
	}
</style>
