init:
  just download-models
  scripts/bootstrap_db.sh

web-dev:
  cd web && pnpm dev --host 0.0.0.0

download-models:
  mkdir -p models
  wget https://huggingface.co/LLukas22/mpt-7b-ggml/resolve/main/mpt-7b-instruct-q4_0.bin -O models/mpt-7b-chat-q4_0.bin
  wget https://huggingface.co/LLukas22/mpt-7b-ggml/resolve/main/mpt-7b-chat-q4_0.bin -O models/mpt-7b-instruct-q4_0.bin

QDRANT_HTTP_PORT := env_var_or_default('QDRANT_HTTP_PORT', '6333')
QDRANT_GRPC_PORT := env_var_or_default('QDRANT_GRPC_PORT', '6334')

run-qdrant:
  docker run \
    -p {{QDRANT_HTTP_PORT}}:6333 \
    -p {{QDRANT_GRPC_PORT}}:6334 \
    -v {{justfile_directory()}}/qdrant/storage:/qdrant/storage \
    --restart unless-stopped \
    --name maiven-qdrant \
    -d \
    qdrant/qdrant
