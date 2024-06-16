<h1 align="center">:writing_hand: oreq</h1>

<h2 align="center"><em>OpenAPI Request Prompts</em></h2>

[![](https://img.shields.io/github/license/uzimaru0000/oreq?style=for-the-badge)](https://github.com/uzimaru0000/oreq/blob/master/LICENSE)
[![](https://img.shields.io/github/v/release/uzimaru0000/oreq?style=for-the-badge)](https://github.com/uzimaru0000/oreq/releases/latest)
![](https://img.shields.io/github/downloads/uzimaru0000/oreq/total?style=for-the-badge)

<h3 align="center">The tool for interactively creating curl arguments from OpenAPI.</h4>

## How to use

![demo](./.github/images/demo.gif)

## USAGE
```
oreq [OPTIONS] <SCHEMA>
```

### OPTIONS
```
-b, --base-url <BASE_URL>        Base URL
-H, --headers <HEADERS>
-p, --path <PATH>                Path to request
-X, --request <METHOD>           Method to use
-P, --param <PATH_PARAM>         Path parameters
-q, --query-param <QUERY_PARAM>  Query parameters
-f, --field <FIELD>              Request body
-h, --help                       Print help
-V, --version                    Print version
```

### ARGS
```
<SCHEMA>    OpenAPI schema path
```

## Example

### Basic use case

```bash
$ oreq github.yaml
> Path /repos/{owner}/{repo}
> Method GET
> owner uzimaru0000
> repo oreq
-X GET 'https://api.github.com/repos/uzimaru0000/oreq'
```

### Send a request using curl

```bash
$ oreq github.yaml | xargs curl
> Path /repos/{owner}/{repo}
> Method GET
> owner uzimaru0000
> repo oreq
{
  "id": 736848036,
  "node_id": "R_kgDOK-topA",
  "name": "oreq",
  "full_name": "uzimaru0000/oreq",
  "private": false,
  "owner": {
    "login": "uzimaru0000",
    "id": 13715034,
    "node_id": "MDQ6VXNlcjEzNzE1MDM0",
    "avatar_url": "https://avatars.githubusercontent.com/u/13715034?v=4",
    "gravatar_id": "",
    "url": "https://api.github.com/users/uzimaru0000",
    ....
```

### Read schema from pipe

```bash
curl -s https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.yaml | oreq -
> Path /repos/{owner}/{repo}
> Method GET
> owner uzimaru0000
> repo oreq
-X GET 'https://api.github.com/repos/uzimaru0000/oreq'
```

## WIP :construction:

### TODO

- [ ] Resolve external reference
