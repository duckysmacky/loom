<script lang="ts">
	import { History, Pointer } from '@lucide/svelte';
	import Icon from '$lib/components/ui/Icon.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import DateInput from '$lib/components/ui/DateInput.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import { graph } from '$lib/stores/graph.svelte';
	import { notify } from '$lib/stores/toasts.svelte';
	import { combineDateTime, isoDateToDisplay, toDateInput, toTimeInput } from '$lib/graph/display';
	import type { NodeResponse } from '$lib/types/NodeResponse';

	/**
	 * Small square button next to a Poke button: opens a dialog to log a poke
	 * at a past date and time, for work that was never logged in the moment.
	 */
	let { node }: { node: NodeResponse } = $props();

	let open = $state(false);
	let date = $state('');
	let time = $state('');
	let saving = $state(false);

	const today = () => toDateInput(new Date().toISOString());
	const pokedAt = $derived(combineDateTime(date, time));
	const error = $derived(
		!date || pokedAt === null
			? 'Enter a date and a 24h time like 14:30.'
			: new Date(pokedAt) > new Date()
				? 'A poke can’t be in the future.'
				: ''
	);

	function show() {
		const now = new Date().toISOString();
		date = toDateInput(now);
		time = toTimeInput(now);
		open = true;
	}

	async function save() {
		if (error || pokedAt === null) return;
		saving = true;
		const poked = await graph.poke(node, pokedAt);
		saving = false;
		if (poked) {
			notify(`Poked “${node.title}” on ${isoDateToDisplay(date)} ${time.trim()}`);
			open = false;
		}
	}
</script>

<!-- Keeps clicks (and the dialog's own) from reaching a clickable parent like NodeCard. -->
<!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
<span class="poke-at" onclick={(event) => event.stopPropagation()}>
	<button
		type="button"
		class="square"
		title="Log a poke at a past date and time"
		aria-label="Log a poke at a past date and time"
		onclick={show}
	>
		<Icon icon={History} always />
	</button>

	<Modal {open} onclose={() => (open = false)} label="Log a past poke" width={380}>
		<form
			class="dialog"
			onsubmit={(event) => {
				event.preventDefault();
				save();
			}}
		>
			<div class="label">Log a past poke</div>
			<div class="title">{node.title}</div>

			<div class="fields">
				<label class="field-group">
					<span class="label">Date</span>
					<DateInput bind:value={date} max={today()} label="Date" />
				</label>
				<label class="field-group">
					<span class="label">Time</span>
					<input
						class="field"
						type="text"
						inputmode="numeric"
						placeholder="HH:MM"
						maxlength="5"
						autocomplete="off"
						bind:value={time}
					/>
				</label>
			</div>
			{#if error}<p class="error">{error}</p>{/if}

			<div class="actions">
				<Button variant="quiet" onclick={() => (open = false)}>Cancel</Button>
				<Button variant="primary" type="submit" disabled={saving || !!error} icon={Pointer}
					>Add poke</Button
				>
			</div>
		</form>
	</Modal>
</span>

<style>
	.poke-at {
		display: inline-flex;
		align-self: stretch;
	}

	.square {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		align-self: stretch;
		aspect-ratio: 1;
		min-width: 28px;
		min-height: 28px;
		padding: 0;
		border: var(--border-width-hair) solid var(--line);
		background: var(--surface);
		color: var(--accent);
		transition:
			opacity var(--fast) var(--ease),
			transform var(--fast) var(--ease);
	}

	.square:hover {
		opacity: 0.85;
	}

	.square:active {
		transform: translateY(1px);
	}

	.dialog {
		padding: 20px 22px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.title {
		margin-top: -8px;
		font: 700 20px/1.2 var(--font-display);
	}

	.fields {
		display: flex;
		gap: 12px;
	}

	.field-group {
		display: flex;
		flex-direction: column;
		gap: 8px;
		flex: 1;
		min-width: 0;
	}

	.error {
		margin: -6px 0 0;
		font: 600 12px/1.4 var(--font-display);
		color: var(--warn);
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		padding-top: 14px;
		border-top: var(--border-width-hair) solid var(--line);
	}
</style>
