these are the current guildlines for contributing to ButterflyVR. the repo is still in its early stages so these may change, an announcement will be made in the discord if significant changes are made to avoid confusion.

### general guidelines
1. please use the issue templates when creating issues, unless no suitable template is available
2. please only create issues relevant to the latest version of the game
3. if you encounter a bug that was previously fixed (a regression) please say so and link the original issue for the bug if it is available along with the version the bug was fixed in
4. if you are making a feature request, please ensure the feature is actually desirable. if you are unsure consider opening a [discussion](https://github.com/Butterfly-VR/ButterflyVR/discussions) about the feature
5. likewise if you are changing the implementation of an in development or added feature you should also open a discussion so alternatives can be considered
6. try to make PRs that handle a single issue at once, generally a PR should close exactly one issue but exceptions are not uncommon. if a PR is too big you may be asked to split it into smaller PRs
7. please use ''git pull --rebase'' when updating a fork to avoid merge commits
8. generally you should merge commits in a PR together such that each commit is a stable version, so each version should build and not have any bugs that are fixed in later commits
9. in general be sure to follow the [git style guide](https://github.com/agis/git-style-guide) and keep your commit messages clear and readable
10. if you make a PR that closes a open issue please include the issue number in the description along with one of the github closing keywords. for example "Fixes #123"
11. if existing documentation for a feature, function, or other doc page becomes outdated due to changes made in your PR, ensure you update that documentation in your PR before requesting a merge
