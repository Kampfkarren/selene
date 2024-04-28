# Pre-commit

`pre-commit` allows integration of `selene` into your Git workflow using git hooks. After [installing pre-commit](https://pre-commit.com/#install), add one of the following configurations to your `.pre-commit-config.yaml` file:

* Use the `selene` binary present on the system path (Should be pre-installed):

```yaml
repos:
  - repo: https://github.com/Kampfkarren/selene
    rev: ''
    hooks:
      - id: selene-system
```

* Use `selene` through GitHub releases:

```yaml
repos:
  - repo: https://github.com/Kampfkarren/selene
    rev: ''
    hooks:
      - id: selene-github
```

* Use the `selene` binary present in the `selene` docker image (Since this uses docker, it might take some time to bootstrap and is slower than the other two options):

```yaml
repos:
  - repo: https://github.com/Kampfkarren/selene
    rev: ''
    hooks:
      - id: selene-docker
```

You may see a `warning` being generated when pre-commit runs. To resolve that, set the `rev` key to any selene tag or commit, for e.g. `rev: '0.26.2'`.