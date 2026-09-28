<script lang="ts">
	import { graph } from '$lib/stores/graph.svelte';

	/** Toggle buttons for every topic; `selected` holds the chosen topic ids. */
	let { selected = $bindable() }: { selected: string[] } = $props();

	function toggle(topicId: string) {
		selected = selected.includes(topicId)
			? selected.filter((id) => id !== topicId)
			: [...selected, topicId];
	}
</script>

{#if graph.topics.length}
	<div class="group">
		<span class="label">Topics</span>
		<div class="topics">
			{#each graph.topics as topic (topic.id)}
				<button
					type="button"
					class="topic"
					class:selected={selected.includes(topic.id)}
					aria-pressed={selected.includes(topic.id)}
					onclick={() => toggle(topic.id)}
				>
					{#if topic.color}<span class="swatch" style:background={topic.color}></span>{/if}
					{topic.name}
				</button>
			{/each}
		</div>
	</div>
{/if}

<style>
	.group {
		display: flex;
		flex-direction: column;
		gap: 8px;
		align-items: flex-start;
	}

	.topics {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.topic {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 6px 9px;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		font: 500 10.5px/1 var(--font-mono);
		color: var(--ink-2);
	}

	.topic.selected {
		border-color: var(--ink);
		background: var(--ink);
		color: var(--on-ink);
	}

	.swatch {
		width: 7px;
		height: 7px;
	}
</style>
