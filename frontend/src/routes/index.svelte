<script lang="ts">
	import { goto } from '$app/navigation';

	export let goodreadsUrl: string;
	export let loading: boolean;

	const BACKEND_BASE_URL = 'http://127.0.0.1:8001';

	async function downloadEbook() {
		loading = true;
		await goto(`${BACKEND_BASE_URL}/download/${encodeURIComponent(goodreadsUrl)}`);
	}
</script>

<main>
	<h1>LibReads</h1>

	<p>Paste your Goodreads URL:</p>
	<input bind:value={goodreadsUrl} />
	{#if loading}
		<h2>Preparing ebook...</h2>
	{:else}
		<button on:click={downloadEbook}>Download Mobi</button>
	{/if}
</main>

<style>
	main {
		text-align: center;
		padding: 1em;
		max-width: 240px;
		height: 100%;
		position: relative;
		margin: 0 auto;
	}

	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}

	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
</style>
