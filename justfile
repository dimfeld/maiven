init:
  just download-models

download-models:
  mkdir -p models
  wget https://huggingface.co/LLukas22/mpt-7b-ggml/resolve/main/mpt-7b-instruct-q4_0.bin -O models/mpt-7b-chat-q4_0.bin
  wget https://huggingface.co/LLukas22/mpt-7b-ggml/resolve/main/mpt-7b-chat-q4_0.bin -O models/mpt-7b-instruct-q4_0.bin

