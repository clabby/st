# `st`

`st` is yet-another tool for managing stacked PRs on GitHub.

## Why does this tool exist?

I'm a long-time user and lover of [Graphite](https://github.com/withgraphite). I never quite used the graphite ecosystem
as intended - only the CLI. As the ecosystem expanded, I feared it would become a paid service, and that has happened.
However, I adore the CLI tool, and it's become a central part of my and my team's workflow.
Stacked PRs make code review + unblocking oneself easier, Graphite makes stacked PRs extremely easy to work with,
etc. Graphite also has some nice features with their web interface, slack integration, MacOS menu bar app, that I rarely
if ever used (cool I guess, though!)

My biggest gripe with Graphite is that they decided to disallow opting into the periphery services that cost them money
to operate, making the CLI unusable in organizations without paying $99/month (_for 5 seats_). Because I, nor most other
Graphite lovers I talked to, ever use these parts of the service, I was pretty disappointed to see:

```
ERROR: Your team's Graphite plan has expired. To continue using Graphite in <organization>, upgrade your 
plan at https://app.graphite.dev/settings/billing?org=<organization>
```

Admittedly, this error message rage-prompted the creation of this project. This tool aims to be an entirely client 
side, minified version of the Graphite CLI's featureset. It is not a _service_, it is just a _tool_. It's also free
software - go crazy. If you enjoy using this tool, consider buying me a beer if we ever meet.

> [!NOTE]
>
> This tool is meant for the common-{wo}man working on open source projects, not for enterprise customers.
>
> If you're looking for a more feature-rich ecosystem for stacked PRs that has a support team, product support, 
> andressen-horowitz funding, etc., I do recommend checking out Graphite. They'll actually fix your bugs - I won't.
> The Graphite team is great, and they've built something very special. I just wish I could opt-out of the fancy stuff!
>
> If you're someone who doesn't care about features like AI code review, web interfaces, etc., and you just want 
> to be able to manage PR stacks on GitHub + manage your stacks locally, this may be the tool for you.

## Usage

_TODO_
