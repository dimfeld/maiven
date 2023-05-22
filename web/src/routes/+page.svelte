<script lang="ts">
  import { enhance, applyAction } from '$app/forms';
  import type { SubmitFunction } from '$app/forms';

  export let data;
  export let form;

  const handleChatSubmit: SubmitFunction = ({ cancel, formElement, formData }) => {
    let input = formData.get('input');
    if (!input) {
      cancel();
      return;
    }

    data.messages = [...data.messages, input];

    return async ({ result }) => {
      applyAction(result);
      if (result.type == 'success') {
        formElement.reset();
        chatInput.focus();

        let response = result.data?.response;
        if (response) {
          data.messages = [...data.messages, response];
        }
      }
    };
  };

  let chatInput: HTMLTextAreaElement;
</script>

<div class="container h-full mx-auto flex justify-center items-center">
  <div class="space-y-5">
    {#if form?.error}
      <div class="alert variant-filled-error">
        {form.error}
      </div>
    {/if}
    <form method="POST" use:enhance={handleChatSubmit}>
      <textarea
        bind:this={chatInput}
        name="input"
        class="textarea"
        required
        value={form?.input ?? ''}
      />
      <button type="submit" class="btn variant-filled-primary">Send</button>
    </form>
    {#each data.messages as message}
      <div class="card">
        {message}
      </div>
    {/each}
  </div>
</div>
