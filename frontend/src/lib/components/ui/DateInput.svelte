<script lang="ts">
	import { displayToIsoDate, isoDateToDisplay } from '$lib/graph/display';

	/**
	 * Date field that always reads and writes `dd.mm.yyyy` - the native
	 * `<input type="date">` follows the browser locale and can't be forced.
	 * `value` stays `yyyy-mm-dd` ('' = empty).
	 */
	let {
		value = $bindable(''),
		onchange,
		disabled = false,
		title,
		min,
		label,
		compact = false
	}: {
		value?: string;
		onchange?: (value: string) => void;
		disabled?: boolean;
		title?: string;
		/** Earliest accepted `yyyy-mm-dd`. */
		min?: string;
		label?: string;
		compact?: boolean;
	} = $props();

	let text = $derived(isoDateToDisplay(value));
	let draft = $state<string | null>(null);
	let invalid = $state(false);

	function commit(event: Event & { currentTarget: HTMLInputElement }) {
		const parsed = displayToIsoDate(event.currentTarget.value);
		invalid = parsed === null || (parsed !== '' && min !== undefined && parsed < min);
		draft = null;
		if (invalid) {
			event.currentTarget.value = text;
			return;
		}
		value = parsed as string;
		onchange?.(value);
	}
</script>

<input
	class="field"
	class:compact
	class:invalid
	type="text"
	inputmode="numeric"
	placeholder="dd.mm.yyyy"
	maxlength="10"
	autocomplete="off"
	aria-label={label}
	aria-invalid={invalid}
	{title}
	{disabled}
	value={draft ?? text}
	oninput={(event) => (draft = event.currentTarget.value)}
	onchange={commit}
/>

<style>
	.compact {
		padding: 5px 6px;
		font-size: 12px;
	}

	.invalid {
		border-color: var(--warn);
	}
</style>
