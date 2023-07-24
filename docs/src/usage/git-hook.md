# Git Hook Support

Selene has [githooks](https://git-scm.com/docs/githooks) support thanks to
[pre-commit](https://pre-commit.com) which means you will need to add a
configuration file - `.pre-commit-config.yaml` to your repository. Thereafter
you'll need to install the hooks and `selene` will be run against all recently
changed files before you commit the changes.

This section of the document details the ways you could setup `pre-commit` for
linting your Lua code before each commits.

In the `.pre-commit-config.yaml` file add the following configurations and then
run `pre-commit install --install-hooks` to setup the pre-commit hooks.

```yaml
repos:
    - repo: https://github.com/Kampfkarren/selene
      rev: '' # Add the latest version of Selene here
      hooks:
          - selene # this will use Cargo to compile the binary before usage
          - selene-system # this will use the installed binary on the system
          - selene-docker # this will build a Docker image before usage
```

To ensure `pre-commit` is the latest tagged version of `selene`, run the `pre-commit autoupdate` command.