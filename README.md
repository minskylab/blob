# Blob

<p align="center">
  <img width="460" src="assets/blob.png">
</p>

```
self-reference is all you need
```

Blob is a LLM (Language Large Model) powered tool to create and work in software projects, it's very experimental and I'm not sure if the actual implementation can scale. The main idea is interact with the source code in a "natural way" to accelerate the actual software development procedure.

Actually only support OpenAI GPT-3 as a engine, but we're working in more adapters to make it compatible with more Open Source technology.

## Main Idea

The principal idea of blob is create a **natural language reversible representation of the file structure** of the project, in this way, we can request to GPT-3 to generate a new file structure based on a natural language instructions. I try some representation, but actually the most simple and effective is a tree-like representation of the file structure.

For example, imagine that you have a project in a folder called `playground` with the following structure:

```
playground/
├── source.ts
└── example.tsx
```

And you want to add a new file called `hello.tsx` in the same folder, you can do it with the following command:

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

Or, if you want bootstrap a new entire project with nextjs and typescript, you can do it with the following command:

```bash
blob do "bootstrap a new nextjs project"
```

The result will be:

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

And if you want to put your `pages` folder into your `src` folder, you can do it with the following command:

```bash
blob do "move the pages folder into the src folder"
```

Resulting in:

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
