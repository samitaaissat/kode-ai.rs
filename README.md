# kode-ai.rs

# Kode AI MCP Server

This is an MCP (Model-Context-Provider) server that interfaces with a GitHub repository to scan for documentation files and provide them to an LLM for contextualization.

It is meant to be used as a backend service that can be queried for documentation related to a specific topic or question. Especially useful for devs working with internal tools.

## Features

- Scan GitHub repositories for documentation files (md, mdx, etc.)
- Store documents in a format suitable for LLM contextualization
- Provide tools to get all documentation or find relevant documentation based on a query

## Requirements

- Rust 2024 or later
- GitHub Personal Access Token (PAT) for private repositories


## Usage

### Running the Server
To run the server, you need to set it up with your agent (such as Github Copilot):

#### 1. Build the Project
```bash
cargo build --release
```

#### 2. Run the Server inside copilot

```json
{
    "servers": {
        "kode-ai": {
            "type": "stdio",
            "command": "/path/to/kode-ai-rs/target/release/kode-ai-rs",
            "args": []
        }
    }
}
```

#### 3. Configure the Server
You may provide arguments to the server to configure it. The server will read the configuration from the CLI arguments.

### Example Configuration
```json
{
    "servers": {
        "kode-ai": {
            "type": "stdio",
            "command": "/path/to/kode-ai-rs/target/release/kode-ai-rs",
            "args": [
                "--github-repo", "my-private-repo",
                "--github-token", "your_github_personal_access_token",
                "--github-owner", "your_github_username",
                "--github-subfolder", "./docs"
            ]
        }
    }
}
```

## MCP Tools

The server provides three tools:

### 1. get_all_docs

Get all documentation from the repository.

Input:
```json
{
  "limit": 10
}
```

Note: The `limit` field is optional and limits the number of documents returned.

Output:
```json
{
  "documents": [
    {
      "path": "docs/README.md",
      "title": "Documentation",
      "summary": "This is the main documentation file.",
      "content": "# Documentation\n\nThis is the main documentation file."
    }
  ]
}
```

Note: The output will include multiple documents if available.

### 2. get_document
Get a specific document by its path.
Input:
```json
{
  "path": "docs/installation.md"
}
```
Output:
```json
{
  "document": {
    "path": "docs/installation.md",
    "title": "Installation Guide",
    "summary": "This guide explains how to install the software.",
    "content": "# Installation Guide\n\nThis guide explains how to install the software."
  }
}
```

### 3. find_relevant_docs

Find documentation relevant to a query.

Input:
```json
{
  "query": "How to install",
  "limit": 5
}
```

Note: The `limit` field is optional and limits the number of documents returned.

Output:
```json
{
  "total": 1,
  "returned": 1,
  "documents": [
    {
      "path": "docs/installation.md",
      "title": "Installation Guide",
      "summary": "This guide explains how to install the software.",
      "content": "# Installation Guide\n\nThis guide explains how to install the software."
    }
  ]
}
```

Note: The output will include multiple documents if available, sorted by relevance to the query. If no documents are found, an empty array will be returned with a message.

## License

GPL-3.0 License


    kode-ai.rs - Kode AI MCP Server
    Copyright (C) 2025  Sami Taaissat

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
