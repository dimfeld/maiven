<script lang="ts">
  import { superForm } from 'sveltekit-superforms/client';
  import { fly } from 'svelte/transition';

  export let data;

  const { form, constraints, enhance, errors, delayed } = superForm(data.form, {
    invalidateAll: false,
    resetForm: false,
    taintedMessage: false,
    timeoutMs: 30000,
  });

  function addMessage(role: string, message: string) {
    data.messages = [...data.messages, { role, message }];
  }

  let chatInput: HTMLTextAreaElement;
  let chatSubmit: HTMLButtonElement;
</script>

<div class="container h-full mx-auto flex justify-center items-center px-4">
  <div class="space-y-5">
    {#each data.messages as message}
      <div
        class="card p-2"
        in:fly|local={{
          x: message.role === 'bot' ? 100 : -100,
        }}
      >
        {message.role}: {message.message}
      </div>
    {/each}
    {#if $delayed}
      <div class="p-2">Loading...</div>
    {/if}
    <form
      method="POST"
      use:enhance={{
        onSubmit({ formData, cancel }) {
          const input = formData.get('input');
          if (typeof input !== 'string' || !input.trim().length) {
            cancel();
            chatInput.focus();
            return;
          }
          addMessage('user', input);
        },
        onResult({ result }) {
          if (result.type == 'success') {
            addMessage('bot', result.data?.response);
          }
        },
        onUpdate({ form }) {
          if (form.valid) {
            form.data.input = '';
          }
        },
        onUpdated() {
          setTimeout(() => {
            console.log(document.activeElement);
            if (document.activeElement === chatSubmit || document.activeElement === document.body) {
              chatInput.focus();
            }
          });
        },
      }}
    >
      <textarea
        bind:this={chatInput}
        name="input"
        class="textarea"
        data-invalid={$errors.input}
        bind:value={$form.input}
        on:keydown={(e) => {
          if (e.key === 'Enter' && e.metaKey) {
            e.preventDefault();
            e.currentTarget.form?.requestSubmit();
          }
        }}
      />
      <button bind:this={chatSubmit} type="submit" class="btn variant-filled-primary">Send</button>
    </form>
  </div>
</div>
