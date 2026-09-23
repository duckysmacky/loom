<script lang="ts">
	import Chip from '$lib/components/ui/Chip.svelte';
	import { nodesApi, topicsApi } from '$lib/api/endpoints';
	import { graph } from '$lib/stores/graph.svelte';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	let { node }: { node: NodeResponse } = $props();

	let newTopicName = $state('');

	const attached = $derived(node.topic_ids.flatMap((id) => graph.topicById.get(id) ?? []));
	const available = $derived(graph.topics.filter((topic) => !node.topic_ids.includes(topic.id)));

	function attach(topicId: string) {
		if (topicId) graph.mutate(() => nodesApi.attachTopic(node.id, topicId));
	}

	async function createAndAttach(event: SubmitEvent) {
		event.preventDefault();
		const name = newTopicName.trim();
		if (!name) return;
		const done = await graph.mutate(async () => {
			const topic = await topicsApi.create({ name });
			await nodesApi.attachTopic(node.id, topic.id);
			return topic;
		});
		if (done) newTopicName = '';
	}
</script>

<section>
	<div class="label">Topics</div>
	<div class="chips">
		{#each attached as topic (topic.id)}
			<Chip
				label={topic.name}
				color={topic.color}
				onremove={() => graph.mutate(() => nodesApi.detachTopic(node.id, topic.id))}
			/>
		{:else}
			<span class="none">No topics</span>
		{/each}
	</div>
	<div class="add">
		{#if available.length}
			<select
				class="field"
				aria-label="Attach a topic"
				value=""
				onchange={(event) => {
					attach(event.currentTarget.value);
					event.currentTarget.value = '';
				}}
			>
				<option value="">+ Attach topic…</option>
				{#each available as topic (topic.id)}
					<option value={topic.id}>{topic.name}</option>
				{/each}
			</select>
		{/if}
		<form onsubmit={createAndAttach}>
			<input
				class="field"
				placeholder="New topic…"
				aria-label="New topic name"
				bind:value={newTopicName}
			/>
		</form>
	</div>
</section>

<style>
	.chips {
		margin-top: 10px;
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.none {
		font: 500 12px/1 var(--font-display);
		color: var(--ink-2);
	}

	.add {
		margin-top: 10px;
		display: grid;
		gap: 6px;
	}

	.add .field {
		padding: 8px 10px;
		font-size: 13px;
		background-color: var(--surface);
	}
</style>
