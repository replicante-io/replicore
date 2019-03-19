# Contributing to the Replicante Project
First of all thank you for your time here!  
We hope you choose to contribute to the Replicante Project as every little help counts.

Any kind of contribution is welcomed, not just code.
If you would like to help but you are not a coder, open a
[new issue](https://github.com/replicante-io/replicante/issues/new) or send an email to
[stefano@spogliani.net](stefano@spogliani.net) describing how you could help and we will be in touch!


## Code of Conduct
This project and everyone participating in it is governed by the Replicante Code of Conduct.
By participating, you are expected to uphold this code.
Please report unacceptable behaviour to [conduct@replicante.io](mailto:conduct@replicante.io).


## Reporting security issues
If you discover a security issue, please bring it to our attention right away!

Please **DO NOT** file a public issue, instead send your report privately to
[stefano@spogliani.net](stefano@spogliani.net).

Security reports are greatly appreciated and we will publicly thank you for it.


## Reporting issues
A good report is key to a fast response.
If you are reporting a bug keep in mind we are not aware of how you have configured and are
running Replicante so you will need to include as much information as possible.
If you are requesting a new feature make sure you include as much detail as possible for your
use case.

In any case try your best to fill out the relevant new issue template as much as possible
to help us better understand and address your needs.

### Before reporting an issue

  1. The first thing to do is read the relevant documentation:
     * We would love for Replicante to be so intuitive documentation is useless. but it is not.
     * Maybe your issue is configuration related.
     * Or what you expect is not what the system is meant to do.
  2. Make sure you are in the right place:
     * The Replicante Project has multiple repos.
     * This helps us keep things separate when they should be.
     * All repositories are here: https://github.com/replicante-io
  3. Search among [existing issues](https://github.com/replicante-io/replicante/issues):
     * Someone else may have already reported your problem.
     * If so, follow the conversation with the "Subscribe" button.
     * Feel free to comment if you have additional information you think may be useful.
     * Show your reactions with emojis on the **issue description**.
     * Please **avoid** emoji-only or "+1" comments.
     * If you want an issue to have higher priority add a "+1" reaction to the issue description.

If none of the above provides a solution it is then time to create a
[new issue](https://github.com/replicante-io/replicante/issues/new).
As a reminder: do try your best to **fill out as much** as possible of the relevant
new issue template.


## Contributing with Pull Requests
Pull Requests require effort be put in even **after** the code is written and the pull request opened.

PRs are an iterative processes between the author and the reviewer(s).
Don't get discouraged if your PR is not excepted as is, most PRs require minor corrections,
especially in case of authors that just started contributing.
Remember that you may be writing just one PR or two but we have to maintain the code
for a long time even after you may stop contributing.

For large or important changes, even bug fixes, you may be better off
opening/commenting on an issue stating your problem and suggested approach.
Try to include alternative options to your implementation and explain why those
ideas have been rejected in favour of your final suggestion.
Having the initial conversation in an issue before you write the code may save you some time
and can get the resuling pull request merged in faster.

Once you are ready to open a pull request:

  1. Make sure you are in the right place:
     * The Replicante Project has multiple repos.
     * This helps us keep things separate when they should be.
     * All repositories are here: https://github.com/replicante-io
  2. Search for existing issues and pull requests:
     * Make sure you are not duplicating work.
     * Existing issues may have information that can guide you as you wirte code.
     * Try to reference existing issues that relate to your change.
  3. If the change is small (handful of lines) it is probably safe to submit the pull request.
  4. If the change is large or has potential side-effects open an issue first to discuss your plan.
  5. [Sign your work](#sign-your-work).


### Sign your work
The sign-off is a simple line at the end of the explanation for the patch. Your
signature certifies that you wrote the patch or otherwise have the right to pass
it on as an open-source patch. The rules are pretty simple: if you can certify
the below (from [developercertificate.org](http://developercertificate.org/)):

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.
1 Letterman Drive
Suite D4700
San Francisco, CA, 94129

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.

Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

Then you just add a line to **every** git commit message:

    Signed-off-by: Joe Smith <joe.smith@email.com>

Use your **real name** (sorry, no pseudonyms or anonymous contributions).

If you set your `user.name` and `user.email` git configs, you can sign your
commit automatically with `git commit -s`.


## Style guides
Style guide are a tool to agree among people about what code, or other resource, should look like.
This ensures the project overall remains consistent over time and easier to maintain.

Style guides on resources such as APIs also help ensure a consistent experience for users.


### API
Any API exposed by Replicante must follow this guidelines:

  * Replicante exposes an HTTP, JSON encoded, REST-like API:
    * Each endpoint MUST use follow specific request and response schemas and is
      NOT allowed to arbitrarily change the "shape" of requests/responses.
    * Error responses MUST follow the same schema across the entire ecosystem.
    * Error responses MUST include and `"error": true` attribute.
  * API Endpoints:
    * Should always be intrduced under the `/api/unstable` tree.
    * They can change as needed until they become stable.
    * Once stable endpoints are moved to the latest API version (`/api/v1`, `/api/v2`, ...).
    * Once an endpoint is stable breaking changes are not allowed (adding attributes to a response
      or optional parameters to requests does not count as a breaking change).
    * If a breaking change is need a new version should be added under the unstble tree.
    * When a new version tree is introduced all API endpoints MUST be made available under it.
    * Older API versions can be removed with a breaking change in reasonable time.


### Rust
The aim is to follow the [official rust guidelines](https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md).

In particular code:

  * Must compile without errors or warnings.
  * Must pass clippy checks without errors.
  * Must pass a `rustfmt` check (`rustfmt` runs without needed to make changes).

**NOTE**: sadly the project is not currently in a state where these guides are not followed.


## Attribution
This document is inspired by a collection of other similar documents:

  * [Atom Contributing Guidelines](https://github.com/atom/atom/blob/master/CONTRIBUTING.md)
  * [Docker (Moby) Contributing Guidelines](https://github.com/moby/moby/blob/master/CONTRIBUTING.md)
  * [Ruby on Rails Contributing Guidelines](https://github.com/rails/rails/blob/master/CONTRIBUTING.md)
