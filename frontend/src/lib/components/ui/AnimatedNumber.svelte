<script lang="ts">
	import { Tween } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	import { ms } from '$lib/motion';

	let { value }: { value: number } = $props();

	// Counts up/down to the new value instead of jumping.
	const shown = Tween.of(() => value, { duration: ms(450), easing: cubicOut });
</script>

{Math.round(shown.current)}
