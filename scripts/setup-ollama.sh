#!/usr/bin/env bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default model
DEFAULT_MODEL="llama2"
MODEL="${1:-$DEFAULT_MODEL}"

echo -e "${CYAN}=== Ollama Setup Script ===${NC}"
echo

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null && ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is not installed${NC}"
    echo "Please install Docker from https://docs.docker.com/get-docker/"
    exit 1
fi

# Use docker compose v2 if available, otherwise fall back to docker-compose
COMPOSE_CMD="docker compose"
if ! docker compose version &> /dev/null; then
    COMPOSE_CMD="docker-compose"
fi

echo -e "${BLUE}📦 Starting Ollama container...${NC}"
$COMPOSE_CMD up -d

# Wait for Ollama to be ready
echo -e "${BLUE}⏳ Waiting for Ollama to be ready...${NC}"
MAX_RETRIES=30
RETRY_COUNT=0

while ! curl -s http://localhost:11434/api/tags > /dev/null; do
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
        echo -e "${RED}❌ Ollama failed to start after ${MAX_RETRIES} seconds${NC}"
        echo "Check logs with: $COMPOSE_CMD logs ollama"
        exit 1
    fi
    sleep 1
done

echo -e "${GREEN}✅ Ollama is running!${NC}"
echo

# Check if model is already pulled
echo -e "${BLUE}🔍 Checking for model: ${MODEL}${NC}"
if docker exec athena-ollama ollama list | grep -q "^${MODEL}"; then
    echo -e "${GREEN}✅ Model '${MODEL}' is already available${NC}"
else
    echo -e "${YELLOW}📥 Pulling model: ${MODEL}${NC}"
    echo "This may take a few minutes depending on model size..."
    docker exec athena-ollama ollama pull "$MODEL"
    echo -e "${GREEN}✅ Model '${MODEL}' pulled successfully${NC}"
fi

echo
echo -e "${GREEN}🎉 Setup complete!${NC}"
echo
echo -e "${CYAN}Available commands:${NC}"
echo -e "  ${YELLOW}Run the chatbot:${NC}"
echo "    cargo run --example ollama_chatbot"
echo
echo -e "  ${YELLOW}Pull a different model:${NC}"
echo "    docker exec athena-ollama ollama pull mistral"
echo "    docker exec athena-ollama ollama pull phi"
echo "    docker exec athena-ollama ollama pull codellama"
echo
echo -e "  ${YELLOW}List available models:${NC}"
echo "    docker exec athena-ollama ollama list"
echo
echo -e "  ${YELLOW}Stop Ollama:${NC}"
echo "    $COMPOSE_CMD down"
echo
echo -e "  ${YELLOW}View logs:${NC}"
echo "    $COMPOSE_CMD logs -f ollama"
echo
