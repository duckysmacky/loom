<script lang="ts">
	import { m } from '$lib/paraglide/messages';
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
	 * Small square button to the right of the node view's Poke button: opens a dialog to log a poke
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
			? m.poke_at_invalid()
			: new Date(pokedAt) > new Date()
				? m.poke_at_future()
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
			notify(
				m.poke_at_done({ title: node.title, date: isoDateToDisplay(date), time: time.trim() })
			);
			open = false;
		}
	}
</script>

<!-- Keeps clicks (and the dialog's own) from reaching a clickable parent. -->
<!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
<span class="poke-at" onclick={(event) => event.stopPropagation()}>
	<button
		type="button"
		class="square"
		title={m.poke_at_open()}
		aria-label={m.poke_at_open()}
		onclick={show}
	>
		<Icon icon={History} always />
	</button>

	<Modal {open} onclose={() => (open = false)} label={m.poke_at_title()} width={380}>
		<form
			class="dialog"
			onsubmit={(event) => {
				event.preventDefault();
				save();
			}}
		>
			<div class="label">{m.poke_at_title()}</div>
			<div class="title">{node.title}</div>

			<div class="fields">
				<label class="field-group">
					<span class="label">{m.field_date()}</span>
					<DateInput bind:value={date} max={today()} label={m.field_date()} />
				</label>
				<label class="field-group">
					<span class="label">{m.field_time()}</span>
					<input
						class="field"
						type="text"
						inputmode="numeric"
						placeholder={m.time_placeholder()}
						maxlength="5"
						autocomplete="off"
						bind:value={time}
					/>
				</label>
			</div>
			{#if error}<p class="error">{error}</p>{/if}

			<div class="actions">
				<Button variant="quiet" onclick={() => (open = false)}>{m.action_cancel()}</Button>
				<Button variant="primary" type="submit" disabled={saving || !!error} icon={Pointer}
					>{m.poke_at_submit()}</Button
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
