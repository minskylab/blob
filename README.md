# Blob

Blob is a OpenAI GPT-3 powered tool to bootstrap software projects, it's very experimental and I'm not sure if it will work.

## Main Idea

The principal idea of blob is create a **natural language reversible representation** of the file structure of the project, in this way, we can request to GPT-3 to generate a new file structure based on a natural language instructions. I try some representation, but actually the most simple and effective is a tree-like representation of the file structure, example below.

```
playground/
├── source.ts
└── example.tsx
```

## Usage

The way to use is invoking the `blob` command with the name of the project you want to create, for example:

```bash
blob create my-project
```

This will create a directory called `my-project` with its own context used as input for GPT-3 engine throw OpenAI API.

With your new project, you can invoke the `blob do` and a instruction in natural language indicating a feature or an action you want to do, for example:

```bash
blob do "bootstrap a new nextjs project"
```

```bash
blob do "add some basic components for a design system"
```

```bash
blob do "add a new page to the project and put a big hello world in the center of this page"
```

### Acknowledgements

Actually I use some code from [https://github.com/jacwah/oak](Oak) project to parse the file structure into tree-like. In the future I document this part better.
