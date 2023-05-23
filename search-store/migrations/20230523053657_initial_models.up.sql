INSERT INTO models (id, name, category, params)
  VALUES 
  (1, 'all-mpnet-base-v2', 'bi-encoder',
    jsonb_build_object(
      'code', 'rust-bert',
      'location', 'huggingface:sentence-transformers/all-mpnet-base-v2'
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
  )
  ;

