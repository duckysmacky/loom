<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Pencil, Plus, Trash2 } from '@lucide/svelte';
	import './settings.css';
	import AccentPicker from '$lib/components/ui/AccentPicker.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { topicsApi } from '$lib/api/endpoints';
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
		if (await graph.mutate(() => topicsApi.remove(topicId))) notify(m.topics_deleted({ name }));
	}
</script>

<section class="settings-section">
	<h2>{m.settings_topics()}</h2>
	<p class="explain">
		{m.topics_explain()}
	</p>

	<form class="create" onsubmit={create}>
		<input
			class="field"
			placeholder={m.topics_new_name_placeholder()}
			aria-label={m.topics_new_label()}
			bind:value={newName}
		/>
		<AccentPicker value={newColor} onchange={(color) => (newColor = color)} />
		<Button type="submit" variant="primary" disabled={!newName.trim()} icon={Plus}
			>{m.action_add()}</Button
		>
	</form>

	<ul class="topics">
		{#each graph.topics as topic (topic.id)}
			<li class="topic">
				<span class="dot" style:background={topic.color ?? 'var(--line)'}></span>
				{#if renamingId === topic.id}
					<!-- svelte-ignore a11y_autofocus -->
					<input
						class="field rename"
						aria-label={m.topics_rename({ name: topic.name })}
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
				<span class="count">{m.node_count({ count: nodeCounts.get(topic.id) ?? 0 })}</span>
				<div class="colors">
					<AccentPicker
						value={topic.color}
						size="small"
						onchange={(color) => graph.mutate(() => topicsApi.update(topic.id, { color }))}
					/>
				</div>
				{#if deletingId === topic.id}
					<Button variant="quiet" onclick={() => (deletingId = null)}>{m.action_keep()}</Button>
					<Button variant="primary" onclick={() => remove(topic.id, topic.name)} icon={Trash2}
						>{m.action_delete()}</Button
					>
				{:else}
					<Button
						variant="quiet"
						onclick={() => {
							renamingId = topic.id;
							renameDraft = topic.name;
						}}
						icon={Pencil}>{m.topics_rename_action()}</Button
					>
					<Button variant="quiet" onclick={() => (deletingId = topic.id)} icon={Trash2}
						>{m.action_delete()}</Button
					>
				{/if}
			</li>
		{:else}
			<li class="empty">{m.topics_empty()}</li>
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

	.colors {
		display: flex;
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
