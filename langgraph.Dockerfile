FROM langchain/langgraph-api:3.11

ARG APP_NAME
ENV APP_NAME=${APP_NAME}

# Set working directory
WORKDIR /deps

# Copy only Python libs for shared code between LangGraph agents
# Rust libs are not needed for Python-based LangGraph agents
COPY libs/python/ ./libs/python/

# Copy only the specific app we're building
COPY apps/${APP_NAME}/ ./app/

# Install the specific app and its dependencies directly from its own pyproject.toml
# This includes any shared libs that the app references in its dependencies
RUN PYTHONDONTWRITEBYTECODE=1 uv pip install --system --no-cache-dir -c /api/constraints.txt -e ./app

# Set the LangServe graphs environment variable based on the app structure
# This reads from the langgraph.json configuration in the app directory
# For now, we'll use a generic approach that works with the react_agent structure
ENV LANGSERVE_GRAPHS="{\"agent\": \"/deps/app/src/react_agent/graph.py:graph\"}"

# Ensure user deps didn't inadvertently overwrite langgraph-api
RUN mkdir -p /api/langgraph_api /api/langgraph_runtime /api/langgraph_license && \
    touch /api/langgraph_api/__init__.py /api/langgraph_runtime/__init__.py /api/langgraph_license/__init__.py

RUN PYTHONDONTWRITEBYTECODE=1 uv pip install --system --no-cache-dir --no-deps -e /api

# Remove pip and uv from the final image for security
RUN pip uninstall -y pip setuptools wheel && \
    rm -rf /usr/local/lib/python*/site-packages/pip* /usr/local/lib/python*/site-packages/setuptools* /usr/local/lib/python*/site-packages/wheel* && \
    find /usr/local/bin -name "pip*" -delete || true

# pip removal for wolfi
RUN rm -rf /usr/lib/python*/site-packages/pip* /usr/lib/python*/site-packages/setuptools* /usr/lib/python*/site-packages/wheel* && \
    find /usr/bin -name "pip*" -delete || true

RUN uv pip uninstall --system pip setuptools wheel && rm /usr/bin/uv /usr/bin/uvx

# Set working directory to the app
WORKDIR /deps/app

# Security and metadata labels
LABEL \
    org.opencontainers.image.title="${APP_NAME}" \
    org.opencontainers.image.description="LangGraph agent application" \
    security.non-root="false" \
    security.langgraph-api="true" \
    security.python-app="true"

# The base image already sets up the proper entrypoint for LangGraph API
