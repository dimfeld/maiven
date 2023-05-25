INSERT INTO models (id, name, category, params)
  VALUES 
  (1, 'all-mpnet-base-v2', 'bi-encoder',
    jsonb_build_object(
      'code', 'rust-bert',
      'location', 'huggingface:diptanuc/all-mpnet-base-v2'
    )
  ),
  (2, 'ms-marco-MiniLM-L-6-v2', 'cross-encoder',
    jsonb_build_object(
      'code', 'rust-bert',
      'location', 'huggingface:cross-encoder/ms-marco-MiniLM-L-6-v2'
    )
  ),
  (3, 'gpt-3.5-turbo', 'chat',
    jsonb_build_object('code', 'openai-chat')
  ),
  (4, 'text-davinci-003', 'complete',
    jsonb_build_object('code', 'openai-completions')
  ),
  (5, 'mpt7b-chat-q4_0', 'chat',
    jsonb_build_object(
      'code', 'ggml',
      'model', 'mpt',
      'location', 'https://huggingface.co/LLukas22/mpt7b-ggml/resolve/main/mpt-7b-chat-q4_0-ggjt.bin'
    )
  ),
  (6, 'mpt7b-instruct-q4_0', 'instruct',
    jsonb_build_object(
      'code', 'ggml',
      'model', 'mpt',
      'location', 'https://huggingface.co/LLukas22/mpt7b-ggml/resolve/main/mpt-7b-instruct-q4_0-ggjt.bin'
    )
  ),
  (7, 'RedPajama-INCITE-Chat-3B', 'chat',
    jsonb_build_object(
      'code', 'ggml',
      'model', 'gpt-neox',
      'location', 'https://huggingface.co/keldenl/RedPajama-INCITE-Chat-3B-v1-GGML/resolve/main/rp-chat-3b-v1-ggml-model-q4_0.bin'
    )
  ),
  (8, 'RedPajama-INCITE-Chat-7B', 'chat',
    jsonb_build_object(
      'code', 'ggml',
      'model', 'gpt-neox',
      'location', 'https://huggingface.co/keldenl/RedPajama-INCITE-Chat-7B-v0.1-GGML/resolve/main/ggml-model-q4_0.bin'
    )
  ),
  (9, 'RedPajama-INCITE-Instruct-3B', 'instruct',
    jsonb_build_object(
      'code', 'ggml',
      'model', 'gpt-neox',
      'location', 'https://huggingface.co/keldenl/RedPajama-INCITE-Instruct-3B-v1-GGML/resolve/main/rp-instruct-3b-v1-ggml-model-q4_0.bin'
    )
  ),
  (10, 'RedPajama-INCITE-Instruct-7B', 'instruct',
    jsonb_build_object(
      'code', 'ggml',
      'model', 'gpt-neox',
      'location', 'https://huggingface.co/keldenl/RedPajama-INCITE-Instruct-7B-v0.1-GGML/resolve/main/rp-instruct-7B-v0.1-ggml-model-q4_0.bin'
    )
  ),
  (11, 'all-MiniLM-L6-v2', 'bi-encoder',
    jsonb_build_object(
      'code', 'rust-bert',
      'location', 'rust-bert',
    )
  )
  ;

