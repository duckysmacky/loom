<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { login } from '$lib/stores/session.svelte';
	import AuthForm from '../AuthForm.svelte';
</script>

<svelte:head><title>{m.auth_sign_in()} · Loom</title></svelte:head>

<AuthForm
	heading={m.auth_sign_in()}
	submitLabel={m.auth_sign_in()}
	passwordAutocomplete="current-password"
	onsubmit={login}
>
	{#snippet footer()}{m.auth_no_account()} <a href="/signup">{m.auth_create_one()}</a>{/snippet}
</AuthForm>
