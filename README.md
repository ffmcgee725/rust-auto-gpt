# Auto GPT Code Generator

An auto GPT project that takes a user prompt to generate a full stack application built with Rust and OpenAI's API.

## Table of Contents

- [Introduction](#introduction)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Usage](#usage)
- [Running Tests](#running-tests)
- [Building for Production](#building-for-production)
- [Contributing](#contributing)
- [License](#license)

## Introduction

This project leverages OpenAI's API to generate a full stack application based on a user prompt. The backend is built with Rust, and the project aims to provide a solid foundation for rapid full stack application development.

## Features

- Generates a backend in Rust
- Interacts with OpenAI's GPT-4 model
- Modular codebase for easy extension
- CLI for user prompts and interaction

## Prerequisites

- Rust (https://www.rust-lang.org/tools/install)
- OpenAI API key (https://beta.openai.com/signup/)

## Installation

1. **Clone the repository:**

```sh
git clone https://github.com/jc992/rust-auto-gpt.git
cd rust-auto-gpt
```

2. **Copy .env.sample to .env and configure your OpenAI API key:**

```sh
cp .env.sample .env
```

Edit the .env file to include your OpenAI API key and organization ID:

```makefile
OPEN_AI_KEY=your_openai_api_key
OPEN_AI_ORG=your_openai_org_id
```

For more information on obtaining an API key, visit the [OpenAI documentation](https://platform.openai.com/docs/overview).

## Usage

To interact with the application, simply run it and follow the terminal prompts to specify the type of website you want to build. The application will use different agents to generate the necessary code.

The generated backend code can be found in `src/templates/web_server/src/main.rs`.
The generated frontend code can be found in ``src/templates/web_app/src`.

## Running Tests

You can run isolated tests on each existing agent. Be aware that running these tests will cost credits as they make queries to OpenAI's API.

To run the tests:

```sh
cargo test
```

## Building for Production

To build a production/release version of the application:

```sh
cargo build --release
```

The optimized binary will be available in the `target/release` directory.

## Contributing

We welcome contributions! Create pull requests with your suggested changes!

## License

This project is licensed under the MIT License. See the LICENSE file for more details.
