import { env } from '$env/dynamic/private';
import { fail } from '@sveltejs/kit';
import ky from 'ky-universal';
import { z } from 'zod';
import { superValidate } from 'sveltekit-superforms/server';

const OPENAI_URL = env.OPENAI_URL || 'https://api.openai.com/v1/chat/completions';

const chatSchema = z.object({
  input: z.string().nonempty(),
});

export async function load() {
  return {
    form: await superValidate(chatSchema),
    messages: [] as { role: 'user' | 'bot'; message: string }[],
  };
}

export const actions = {
  default: async ({ fetch, request }) => {
    const form = await superValidate(request, chatSchema);

    if (!form.valid) {
      return fail(400, { form });
    }

    const response = await ky(OPENAI_URL, {
      fetch,
      method: 'POST',
      headers: {
        Authorization: `Bearer ${env.OPENAI_TOKEN}`,
      },
      timeout: 30000,
      json: {
        model: 'gpt-3.5-turbo',
        messages: [
          {
            role: 'user',
            content: form.data.input,
          },
        ],
      },
    }).json();

    return {
      form,
      response: response.choices[0].message.content as string,
    };
  },
};
