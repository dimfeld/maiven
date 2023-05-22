import { env } from '$env/dynamic/private';
import { fail } from '@sveltejs/kit';
import ky from 'ky-universal';

const OPENAI_URL = env.OPENAI_URL || 'https://api.openai.com/v1/chat/completions';

export async function load() {
  return {
    messages: [] as string[],
  };
}

export const actions = {
  default: async ({ fetch, request }) => {
    const data = await request.formData();
    const input = data.get('input')?.toString().trim();

    if (!input) {
      return fail(400, { error: 'Chat is empty' });
    }

    const response = await ky(OPENAI_URL, {
      fetch,
      method: 'POST',
      headers: {
        Authorization: `Bearer ${env.OPENAI_TOKEN}`,
      },
      json: {
        model: 'gpt-3.5-turbo',
        messages: [
          {
            role: 'user',
            content: input,
          },
        ],
      },
    }).json();

    return {
      response: response.choices[0].message.content as string,
    };
  },
};
