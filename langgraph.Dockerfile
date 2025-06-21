# Reusable Dockerfile for LangGraph applications
# Usage: docker build -f langgraph.Dockerfile --build-arg APP_NAME=agent -t agent:latest .

FROM docker.io/langchain/langgraph-api:3.11

# Build argument for the app name
ARG APP_NAME
ENV APP_NAME=${APP_NAME}

# Validate APP_NAME is provided
RUN test -n "$APP_NAME" || (echo "APP_NAME build argument is required" && exit 1)

# Set working directory
WORKDIR /app

# Copy all pre-built wheel files from individual project dist directories
COPY libs/python/shared/dist/*.whl /tmp/wheels/
COPY libs/python/services/dist/*.whl /tmp/wheels/
COPY apps/zerg/*/dist/*.whl /tmp/wheels/

# Install the wheels using uv (without constraints to avoid conflicts)
RUN PYTHONDONTWRITEBYTECODE=1 uv pip install --system --no-cache-dir /tmp/wheels/*.whl

# Copy the specific app source code (needed for LangGraph to find the graph)
COPY apps/zerg/${APP_NAME} /app/

# Clean up wheel files
RUN rm -rf /tmp/wheels

# Set the LangGraph configuration dynamically based on APP_NAME
# This follows the pattern from langgraph.json: "agent": "react_{APP_NAME}:graph"
ENV LANGSERVE_GRAPHS="{\"agent\": \"react_${APP_NAME}:graph\"}"

# Expose the port
EXPOSE 8000

# The base image already has the correct entrypoint
