init:
  just download-models
  scripts/bootstrap_db.sh

web-dev:
  cd web && pnpm dev --host 0.0.0.0

download-models:
  mkdir -p models
  wget https://huggingface.co/LLukas22/mpt-7b-ggml/resolve/main/mpt-7b-instruct-q4_0.bin -O models/mpt-7b-chat-q4_0.bin
  wget https://huggingface.co/LLukas22/mpt-7b-ggml/resolve/main/mpt-7b-chat-q4_0.bin -O models/mpt-7b-instruct-q4_0.bin

