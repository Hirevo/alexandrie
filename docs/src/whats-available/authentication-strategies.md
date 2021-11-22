Authentication Strategies
=========================

Alexandrie supports authenticating using various methods and services.  

Here are the details about the various means of authentication you can use in Alexandrie.

> All of these authentication means currently only apply to the frontend.  
> The programmatic API for Alexandrie already uses its own token system for authorization.  

Local
-----

This strategy is the regular email/password combination flow that is already in place, using the input forms.  
This can be disabled, in case you want to exclusively use an alternative authentication strategy, for example.

Configuration:

```toml
# Omitting this entire section from the configuration counts as being disabled.
[frontend.auth.local]
# Whether to enable the use of this strategy.
enable = true
# Whether to allow the registration of new users using this strategy.
allow_registration = true
```

GitHub
------

This strategy uses OAuth 2 to authenticate the user using its GitHub account.  
Filters on who gets authorized can be added based on organization or team membership.  

Configuration:

```toml
# Omitting this entire section from the configuration counts as being disabled.
[frontend.auth.github]
# Whether to enable the use of this strategy.
enable = true
# The client ID of the GitHub OAuth App to use.
client_id = "GITHUB_OAUTH_CLIENT_ID"
# The client secret of the GitHub OAuth App to use.
client_secret = "GITHUB_OAUTH_CLIENT_SECRET"
# The organizations of which membership in one of them is required to be authorized.
# Omit `allowed_organizations` to not require any organization membership.
allowed_organizations = [
    # Being a member of this organization will be sufficient to be authorized.
    { name = "ORG_NAME_1" },
    # But being a member of this one additionally requires membership in one of the specified teams withing that organization.
    { name = "ORG_NAME_2", allowed_teams = ["TEAM_NAME"] },
]
# Whether to allow the registration of new users using this strategy.
allow_registration = true
```

GitLab
------

This uses OAuth 2 to authenticate the user using its GitLab account.  
The remote instance can either be the [public one](https://gitlab.com) or a private instance.  
Filters on who gets authorized can be added based on group membership.  

Configuration:

```toml
# Omitting this entire section from the configuration counts as being disabled.
[frontend.auth.gitlab]
# Whether to enable the use of this strategy.
enable = true
# The client ID of the GitLab OAuth App to use.
client_id = "GITLAB_OAUTH_CLIENT_ID"
# The client secret of the GitLab OAuth App to use.
client_secret = "GITLAB_OAUTH_CLIENT_SECRET"
# The groups of which membership in one of them is required to be authorized.
# Omit `allowed_groups` to not require any group membership.
allowed_groups = [
    "GROUP_NAME_1",
    # subgroups are specified by their full paths, like this.
    "GROUP_NAME_2/SUBGROUP_NAME_1",
]
# Whether to allow the registration of new users using this strategy.
allow_registration = true
```
