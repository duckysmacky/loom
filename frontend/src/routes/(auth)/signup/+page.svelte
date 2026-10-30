<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { signup } from '$lib/stores/session.svelte';
	import AuthForm from '../AuthForm.svelte';
</script>

<svelte:head><title>{m.auth_create_account()} · Loom</title></svelte:head>

<AuthForm
	heading={m.auth_create_account()}
	submitLabel={m.auth_create_account()}
	passwordAutocomplete="new-password"
	onsubmit={signup}
>
	{#snippet footer()}
		{m.auth_have_account()} <a href="/login">{m.auth_sign_in()}</a>
		<br />{m.auth_password_rule()}
	{/snippet}
</AuthForm>
