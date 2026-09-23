import { redirect } from '@sveltejs/kit';
import { prefs } from '$lib/stores/prefs.svelte';

export function load() {
	redirect(307, `/board/${prefs.defaultBoardView}`);
}
