<script lang="ts">
	import './settings.css';
	import Button from '$lib/components/ui/Button.svelte';
	import { topicsApi } from '$lib/api/endpoints';
	import { ACCENT_PALETTE } from '$lib/graph/display';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify } from '$lib/stores/toasts.svelte';

	let newName = $state('');
	let newColor = $state<string | null>(null);
	let renamingId = $state<string | null>(null);
	let renameDraft = $state('');
	let deletingId = $state<string | null>(null);

	const nodeCounts = $derived(
		new Map(
			graph.topics.map((topic) => [
				topic.id,
				graph.nodes.filter((node) => node.topic_ids.includes(topic.id)).length
			])
		)
	);

	async function create(event: SubmitEvent) {
		event.preventDefault();
		const name = newName.trim();
		if (!name) return;
		const created = await graph.mutate(() => topicsApi.create({ name, color: newColor }));
		if (created) {
			newName = '';
			newColor = null;
		}
	}

	async function rename(topicId: string) {
		const name = renameDraft.trim();
		renamingId = null;
		if (name) await graph.mutate(() => topicsApi.update(topicId, { name }));
	}

	async function remove(topicId: string, name: string) {
		deletingId = null;
		if (await graph.mutate(() => topicsApi.remove(topicId))) notify(`Deleted topic “${name}”`);
	}
</script>

<section class="settings-section">
	<h2>Topics & tags</h2>
	<p class="explain">
		Free-form tags for filtering and grouping. They never block anything - that's what connections
		are for.
	</p>

	<form class="create" onsubmit={create}>
		<input
			class="field"
			placeholder="New topic name…"
			aria-label="New topic name"
			bind:value={newName}
		/>
		<div class="swatches">
			{#each ACCENT_PALETTE as color (color)}
				<button
					type="button"
					class="swatch"
					class:chosen={newColor === color}
					style:background={color}
					aria-label="Color {color}"
					aria-pressed={newColor === color}
					onclick={() => (newColor = newColor === color ? null : color)}
				></button>
			{/each}
		</div>
		<Button type="submit" variant="primary" disabled={!newName.trim()}>Add</Button>
	</form>

	<ul class="topics">
		{#each graph.topics as topic (topic.id)}
			<li class="topic">
				<span class="dot" style:background={topic.color ?? 'var(--line)'}></span>
				{#if renamingId === topic.id}
					<!-- svelte-ignore a11y_autofocus -->
					<input
						class="field rename"
						aria-label="Rename {topic.name}"
						autofocus
						bind:value={renameDraft}
						onblur={() => rename(topic.id)}
						onkeydown={(event) => {
							if (event.key === 'Enter') event.currentTarget.blur();
							if (event.key === 'Escape') renamingId = null;
						}}
					/>
				{:else}
					<span class="name">{topic.name}</span>
				{/if}
				<span class="count">{nodeCounts.get(topic.id)} nodes</span>
				<div class="colors">
					{#each ACCENT_PALETTE as color (color)}
						<button
							type="button"
							class="swatch small"
							class:chosen={topic.color === color}
							style:background={color}
							aria-label="Set {topic.name} color {color}"
							onclick={() =>
								graph.mutate(() =>
									topicsApi.update(topic.id, { color: topic.color === color ? null : color })
								)}
						></button>
					{/each}
				</div>
				{#if deletingId === topic.id}
					<Button variant="quiet" onclick={() => (deletingId = null)}>Keep</Button>
					<Button variant="primary" onclick={() => remove(topic.id, topic.name)}>Delete</Button>
				{:else}
					<Button
						variant="quiet"
						onclick={() => {
							renamingId = topic.id;
							renameDraft = topic.name;
						}}>Rename</Button
					>
					<Button variant="quiet" onclick={() => (deletingId = topic.id)}>Delete</Button>
				{/if}
			</li>
		{:else}
			<li class="empty">No topics yet.</li>
		{/each}
	</ul>
</section>

<style>
	.create {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	.create .field {
		flex: 1;
		min-width: 180px;
	}

	.swatches,
	.colors {
		display: flex;
		gap: 4px;
	}

	.swatch {
		width: 18px;
		height: 18px;
		padding: 0;
		border: var(--border-width) solid transparent;
	}

	.swatch.small {
		width: 13px;
		height: 13px;
		border-width: 1.5px;
	}

	.swatch.chosen {
		border-color: var(--ink);
	}

	.topics {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
	}

	.topic {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
		padding: 10px 0;
		border-top: var(--border-width-hair) solid var(--line);
	}

	.dot {
		width: 9px;
		height: 9px;
		flex: none;
	}

	.name {
		font: 700 13.5px/1.2 var(--font-display);
		min-width: 120px;
	}

	.rename {
		width: 180px;
		padding: 5px 8px;
	}

	.count {
		font: 500 10.5px/1 var(--font-mono);
		color: var(--ink-2);
		margin-right: auto;
	}

	.empty {
		padding: 12px 0;
		color: var(--ink-2);
		border-top: var(--border-width-hair) solid var(--line);
	}
</style>
