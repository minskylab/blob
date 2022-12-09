# Blob

Blob is a OpenAI GPT-3 powered tool to bootstrap software projects, it's very experimental and I'm not sure if it will work.

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
blob do "add some basic component for a design system"
```

```bash
blob do "add a new page to the project and put a big hello world in the center of this page"
```

### Acknowledgements

Actually I use some code from [https://github.com/jacwah/oak](Oak) project. In the future I document it better.
