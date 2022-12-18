# Blob

<p align="center">
  <img width="460" src="assets/blob.png">
</p>

> Self-reference is all you need

## Definition

Blob is a powerful tool that uses language large models (LLMs) to assist in the creation and maintenance of software projects. It is designed to provide a more intuitive and efficient way to work with source code, enabling developers to focus on their ideas and logic rather than the technicalities of programming. Blob is still in the experimental phase and may not yet be able to handle large-scale projects, but we are actively working on improving its capabilities and scalability.

Currently, Blob utilizes OpenAI's GPT-3 as its underlying engine, but we are also exploring the integration of other open source technologies through the use of adapters. Our ultimate goal is to make Blob a versatile and adaptable tool that can be used in a wide range of projects and environments.

## ⚠️ Warning

Please note that Blob is currently under development and is not yet suitable for production use. While you are welcome to try it out and provide feedback, we caution that it may have an incomplete implementation and may not function as intended. Our top priorities at this time are to improve the documentation and release stable binaries. Please check back with us for updates on the progress of these efforts.

## Main Idea

The main idea behind Blob is to create a **natural language reversible representation of the file structure** of a software project. This way, we can ask GPT-3 to generate a new file structure based on natural language instructions. While we have tried different representations, the most simple and effective so far is a tree-like representation of the file structure.

For example, imagine you have a project in a folder called `playground` with the following structure:

```
playground/
├── source.ts
└── example.tsx
```

To add a new file called `hello.tsx` to the same folder, you can use the following command:

```bash
blob do "add a new file called hello.tsx"
```

The result will be:

```
playground/
├── source.ts
├── example.tsx
└── hello.tsx
```

Or, if you want to bootstrap a new Next.js project with TypeScript, you can do so with the following command:

```bash
blob do "bootstrap a new nextjs project"
```

This will result in the following file structure:

```
nextjs-starter/
├── .gitignore
├── README.md
├── package.json
├── pages
│   ├── _app.tsx
│   ├── _document.tsx
│   ├── index.tsx
│   └── about.tsx
├── public
│   ├── favicon.ico
│   ├── logo.png
│   └── robots.txt
├── src
│   ├── components
│   │   ├── header.tsx
│   │   └── layout.tsx
│   └── styles
│       └── global.css
└── tsconfig.json
```

To move the pages folder into the src folder, use the following command:

```bash
blob do "move the pages folder into the src folder"
```

This will result in the following file structure:

```
nextjs-starter/
├── .gitignore
├── README.md
├── package.json
├── public
│   ├── favicon.ico
│   ├── logo.png
│   └── robots.txt
├── src
│   ├── pages
│   │   ├── _app.tsx
│   │   ├── _document.tsx
│   │   ├── index.tsx
│   │   └── about.tsx
│   ├── components
│   │   ├── header.tsx
│   │   └── layout.tsx
│   └── styles
│       └── global.css
└── tsconfig.json
```

## Usage

You only need use the `blob` binary to interact with your project. You can use the `--help` flag to see all the available commands.

```bash
blob --help
```

By default, the current directory is assumed as the context, but you can use the `--path`(`-p`) flag to specify a different directory.

To perform a specific action or feature, you can use the `blob do` command followed by a natural language instruction. For example:

```bash
blob do "bootstrap a new nextjs project"
```

```bash
blob do "add some basic components for a design system"
```

```bash
blob do "add a new page to the project and put a big hello world in the center of this page"
```
